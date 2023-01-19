use piet_common::{kurbo::Point, *};
use std::ops::Range;
const SHY: char = '\u{AD}';

pub trait BreakShyWithDash: RenderContext {
    fn new_text_layout(
        &mut self,
        text: impl TextStorage,
    ) -> BreakingTextLayoutBuilder<
        <<Self as piet_common::RenderContext>::Text as Text>::TextLayoutBuilder,
    >;

    fn draw_breaking_text(
        &mut self,
        layout: &BreakingTextLayout<Self::TextLayout>,
        pos: impl Into<Point>,
    );
}

impl<T: RenderContext> BreakShyWithDash for T {
    fn new_text_layout(
        &mut self,
        text: impl TextStorage,
    ) -> BreakingTextLayoutBuilder<
        <<T as piet_common::RenderContext>::Text as piet_common::Text>::TextLayoutBuilder,
    > {
        let dashes = text
            .as_str()
            .split(SHY)
            .map(|_| Some(self.text().new_text_layout("-")))
            .collect();

        BreakingTextLayoutBuilder::new(self.text().new_text_layout(text), dashes)
    }

    fn draw_breaking_text(
        &mut self,
        text: &BreakingTextLayout<Self::TextLayout>,
        pos: impl Into<Point>,
    ) {
        let border = pos.into();

        for (dash, offset) in &text.dashes {
            self.draw_text(dash, border + (offset.x, offset.y));
        }

        self.draw_text(&text.inner, border);
    }
}

pub struct BreakingTextLayoutBuilder<T: TextLayoutBuilder> {
    inner: T,
    dashes: Vec<Option<T>>,
    default_attributes: Vec<TextAttribute>,
    attributes: Vec<(Range<usize>, TextAttribute)>,
}

impl<T: TextLayoutBuilder> BreakingTextLayoutBuilder<T> {
    fn new(inner: T, dashes: Vec<Option<T>>) -> Self {
        Self {
            inner,
            dashes,
            default_attributes: Vec::new(),
            attributes: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct BreakingTextLayout<T: TextLayout> {
    inner: T,
    dashes: Vec<(T, Point)>,
}

impl<T: TextLayout> TextLayout for BreakingTextLayout<T> {
    fn size(&self) -> kurbo::Size {
        self.inner.size()
    }

    fn trailing_whitespace_width(&self) -> f64 {
        self.inner.trailing_whitespace_width()
    }

    fn image_bounds(&self) -> kurbo::Rect {
        self.inner.image_bounds()
    }

    fn text(&self) -> &str {
        self.inner.text()
    }

    fn line_text(&self, line_number: usize) -> Option<&str> {
        self.inner.line_text(line_number)
    }

    fn line_metric(&self, line_number: usize) -> Option<LineMetric> {
        self.inner.line_metric(line_number)
    }

    fn line_count(&self) -> usize {
        self.inner.line_count()
    }

    fn hit_test_point(&self, point: Point) -> HitTestPoint {
        self.inner.hit_test_point(point)
    }

    fn hit_test_text_position(&self, idx: usize) -> HitTestPosition {
        self.inner.hit_test_text_position(idx)
    }
}

impl<T: TextLayoutBuilder> TextLayoutBuilder for BreakingTextLayoutBuilder<T> {
    type Out = BreakingTextLayout<T::Out>;

    fn max_width(self, width: f64) -> Self {
        Self {
            inner: self.inner.max_width(width),
            ..self
        }
    }

    fn alignment(self, alignment: TextAlignment) -> Self {
        Self {
            inner: self.inner.alignment(alignment),
            ..self
        }
    }

    fn default_attribute(mut self, attribute: impl Into<TextAttribute>) -> Self {
        let attribute = attribute.into();

        self.attributes.push((0..usize::MAX, attribute.clone()));

        Self {
            inner: self.inner.default_attribute(attribute),
            ..self
        }
    }

    fn range_attribute(
        self,
        range: impl std::ops::RangeBounds<usize>,
        attribute: impl Into<TextAttribute>,
    ) -> Self {
        Self {
            inner: self.inner.range_attribute(range, attribute),
            ..self
        }
    }

    fn build(mut self) -> Result<Self::Out, Error> {
        let inner = self.inner.build()?;

        Ok(BreakingTextLayout {
            dashes: {
                let mut result = Vec::new();

                // Place a dash at the end of each line that ends with a SHY character.
                let mut line_number = 0;
                let mut shys = 0;
                while let (Some(line), Some(metric)) =
                    (inner.line_text(line_number), inner.line_metric(line_number))
                {
                    if let Some(last) = line.as_bytes().last() {
                        if *last == SHY as u8 {
                            let offset = metric.end_offset - 2;
                            let hit = inner.hit_test_text_position(offset);

                            let layout = self.dashes[shys].take().unwrap();

                            let layout = self
                                .default_attributes
                                .iter()
                                .fold(layout, |acc, curr| acc.default_attribute(curr.clone()));

                            let layout =
                                self.attributes
                                    .iter()
                                    .fold(layout, |acc, (range, attribute)| {
                                        acc.range_attribute(range.clone(), attribute.clone())
                                    });

                            result.push((layout.build()?, hit.point + (0.0f64, -metric.baseline)));

                            shys += 1;
                        }
                        line_number += 1;
                    }
                }

                result
            },
            inner,
        })
    }
}
