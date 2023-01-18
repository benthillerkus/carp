use carp::{
    artifact::{Artifact, Content},
    Side,
};
use color_eyre::Result;
use std::path::{Path, PathBuf};
use tts_external_api::ExternalEditorApi;

pub fn spawn_deck(
    api: &ExternalEditorApi,
    deck: &[Artifact<PathBuf>],
    position: (f32, f32, f32),
) -> Result<()> {
    if deck.is_empty() {
        return Ok(());
    }

    let backs = deck
        .iter()
        .filter(|artifact| artifact.side == Side::Back)
        .cycle();

    for (front, back) in deck
        .iter()
        .filter(|artifact| artifact.side == Side::Front)
        .zip(backs)
    {
        let _ = api.execute(spawn_card_or_deck_tts(
            position,
            &front.data,
            &back.data,
            front.content,
            front.aspect_ratio.map_or(false, |a| a.is_landscape()),
            true,
        ))?;
    }
    Ok(())
}

fn spawn_card_or_deck_tts(
    position: (f32, f32, f32),
    face: &Path,
    back: &Path,
    content: Content,
    sideways: bool,
    back_is_hidden: bool,
) -> String {
    let mut face = face.display().to_string().replace('\\', "//");
    if !face.starts_with("http") {
        face = format!("file://{face}");
    }
    let mut back = back.display().to_string().replace('\\', "//");
    if !back.starts_with("http") {
        back = format!("file://{back}");
    }

    match content {
        Content::Sheet {
            columns,
            rows,
            total,
        } => {
            format!(
                r#"spawnObject({{
    type = "DeckCustom",
    position = {{{}, {}, {}}},
    snap_to_grid = true,
    callback_function = function(spawned_object)
        spawned_object.setCustomObject({{
            face = "{face}",
            back = "{back}",
            width = {columns},
            height = {rows},
            number = {total},
            sideways = {sideways},
            back_is_hidden = {back_is_hidden},
        }})
    end
}})"#,
                position.0, position.1, position.2,
            )
        }
        Content::Single => format!(
            r#"spawnObject({{
type = "CardCustom",
position = {{{}, {}, {}}},
snap_to_grid = true,
callback_function = function(spawned_object)
    spawned_object.setCustomObject({{
        face = "{face}",
        back = "{back}",
        sideways = {sideways},
    }})
end
}})"#,
            position.0, position.1, position.2,
        ),
    }
}
