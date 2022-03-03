use bevy::{prelude::*, utils::HashSet};

use crate::{typing::TypingTarget, AppState};
pub struct UiPlugin;

#[derive(Component)]
struct ScoreText;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // We need the font to have been loaded for this to work.
        app.add_startup_system(setup)
            .add_system(update_targets)
            .add_system(update_score)
            .add_system_set(SystemSet::on_enter(AppState::Dead).with_system(death_screen));
    }
}

fn death_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("death_screen");
    let container = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(0.),
                    left: Val::Px(0.),
                    ..Default::default()
                },
                size: Size::new(Val::Percent(50.0), Val::Percent(100.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .id();

    let bg = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(70.0), Val::Percent(25.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceEvenly,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            color: Color::BLACK.into(),
            ..Default::default()
        })
        .id();

    let deadtext = commands
        .spawn_bundle(TextBundle {
            style: Style {
                margin: Rect::all(Val::Px(5.0)),
                ..Default::default()
            },
            text: Text {
                sections: vec![TextSection {
                    value: "You Ded".into(),
                    style: TextStyle {
                        font: asset_server.load("Amatic-Bold.ttf"),
                        font_size: 40.,
                        color: Color::WHITE,
                    },
                }],
                ..Default::default()
            },
            ..Default::default()
        })
        .id();

    commands.entity(container).push_children(&[bg]);
    commands.entity(bg).push_children(&[deadtext]);
}

fn update_score(mut query: Query<&mut Text, With<ScoreText>>, score: Res<crate::Score>) {
    if !score.is_changed() {
        return;
    }
    for mut text in query.iter_mut() {
        text.sections[0].value = format!("{}", score.0);
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

    let scoretext = commands
        .spawn_bundle(TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    right: Val::Px(5.0),
                    ..Default::default()
                },
                margin: Rect::all(Val::Px(5.0)),
                ..Default::default()
            },
            text: Text {
                sections: vec![TextSection {
                    value: "0".into(),
                    style: TextStyle {
                        font: asset_server.load("Amatic-Bold.ttf"),
                        font_size: 40.,
                        color: Color::WHITE,
                    },
                }],
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(ScoreText)
        .id();

    commands.entity(root).push_children(&[topbar, bottombar]);
    commands.entity(topbar).push_children(&[toptext, scoretext]);
    commands.entity(bottombar).push_children(&[bottomtext]);
}
