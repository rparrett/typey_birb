use bevy::{prelude::*, utils::HashSet};

use crate::typing::TypingTarget;
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // We need the font to have been loaded for this to work.
        app.add_startup_system(setup).add_system(update_targets);
    }
}

fn update_targets(
    query: Query<(Entity, &TypingTarget), Changed<TypingTarget>>,
    mut text_query: Query<&mut Text>,
) {
    for (entity, target) in query.iter() {
        if let Ok(mut text) = text_query.get_mut(entity) {
            let parts = target.word.split_at(target.index);

            text.sections[0].value = parts.0.into();
            text.sections[1].value = parts.1.into();
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut wordlist: ResMut<crate::typing::WordList>,
) {
    commands.spawn_bundle(UiCameraBundle::default());

    // root node
    let root = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .id();

    let topbar = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Px(50.)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            color: Color::BLACK.into(),
            ..Default::default()
        })
        .id();

    let mut not = HashSet::default();
    let topword = wordlist.find_next_word(&not);
    for c in topword.chars() {
        not.insert(c);
    }

    let toptext = commands
        .spawn_bundle(TextBundle {
            style: Style {
                margin: Rect::all(Val::Px(5.0)),
                ..Default::default()
            },
            text: Text {
                sections: vec![
                    TextSection {
                        value: "".into(),
                        style: TextStyle {
                            font: asset_server.load("Amatic-Bold.ttf"),
                            font_size: 40.,
                            color: Color::GREEN,
                        },
                    },
                    TextSection {
                        value: topword.clone(),
                        style: TextStyle {
                            font: asset_server.load("Amatic-Bold.ttf"),
                            font_size: 40.,
                            color: Color::WHITE,
                        },
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(crate::typing::TypingTarget::new(
            topword.into(),
            crate::Action::BirbUp,
        ))
        .id();

    let bottombar = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Px(50.)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            color: Color::BLACK.into(),
            ..Default::default()
        })
        .id();

    let bottomword = wordlist.find_next_word(&not);
    let bottomtext = commands
        .spawn_bundle(TextBundle {
            style: Style {
                margin: Rect::all(Val::Px(5.0)),
                ..Default::default()
            },
            text: Text {
                sections: vec![
                    TextSection {
                        value: "".into(),
                        style: TextStyle {
                            font: asset_server.load("Amatic-Bold.ttf"),
                            font_size: 40.,
                            color: Color::GREEN,
                        },
                    },
                    TextSection {
                        value: bottomword.clone(),
                        style: TextStyle {
                            font: asset_server.load("Amatic-Bold.ttf"),
                            font_size: 40.,
                            color: Color::WHITE,
                        },
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(crate::typing::TypingTarget::new(
            bottomword,
            crate::Action::BirbDown,
        ))
        .id();

    commands.entity(root).push_children(&[topbar, bottombar]);
    commands.entity(topbar).push_children(&[toptext]);
    commands.entity(bottombar).push_children(&[bottomtext]);
}
