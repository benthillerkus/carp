pub struct Artifact<Format> {
    pub name: String,
    pub data: Format,
}

use crate::Result;

type ArtifactIterator<'a, Format> = Box<dyn Iterator<Item = Result<Artifact<Format>>> + 'a>;

pub struct RenderArtifacts<'a, Format> {
    iter: ArtifactIterator<'a, Format>,
}

impl<'a, Format> Iterator for RenderArtifacts<'a, Format> {
    type Item = Result<Artifact<Format>>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a, Format> RenderArtifacts<'a, Format> {
    pub fn wrap(iter: impl Iterator<Item = Result<Artifact<Format>>> + 'a) -> Self {
        Self {
            iter: Box::new(iter),
        }
    }
}
