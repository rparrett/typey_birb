use bevy::{prelude::*, utils::HashSet};

use crate::{typing::TypingTarget, AppState, FontAssets, GltfAssets};
pub struct UiPlugin;

#[derive(Component)]
struct ScoreText;
#[derive(Component)]
struct StartScreen;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // We need the font to have been loaded for this to work.
        app.add_system(update_targets)
            .add_system(update_score)
            .add_system_set(SystemSet::on_enter(AppState::Dead).with_system(death_screen))
            .add_system_set(
                SystemSet::on_enter(AppState::NotPlaying)
                    .with_system(setup)
                    .with_system(start_screen),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::NotPlaying).with_system(despawn_start_screen),
            );
    }
}

fn despawn_start_screen(mut commands: Commands, query: Query<Entity, With<StartScreen>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn start_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    gltf_assets: Res<GltfAssets>,
) {
    // rival

    commands
        .spawn_bundle((
            Transform::from_xyz(-5.4, 2.3, 4.5)
                .with_scale(Vec3::splat(2.5))
                .with_rotation(Quat::from_euler(EulerRot::XYZ, 0.1, -0.6, -0.7)),
            GlobalTransform::default(),
            StartScreen,
        ))
        .with_children(|parent| {
            parent.spawn_scene(gltf_assets.birb_gold.clone());
        });

    // text

    let container = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(0.),
                    left: Val::Px(0.),
                    ..Default::default()
                },
                size: Size::new(Val::Percent(50.0), Val::Percent(70.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(StartScreen)
        .id();

    let bg = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(70.0), Val::Percent(40.0)),
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::SpaceBetween,
                flex_direction: FlexDirection::ColumnReverse,
                padding: Rect::all(Val::Px(10.0)),
                ..Default::default()
            },
            color: Color::BLACK.into(),
            ..Default::default()
        })
        .id();

    let starttext = commands
        .spawn_bundle(TextBundle {
            style: Style {
                ..Default::default()
            },
            text: Text {
                sections: vec![TextSection {
                    value: "So you want to join the flock, eh?\nYou'll have to beat me first!\nType the word below when you're ready."
                        .into(),
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

    let starttarget = commands
        .spawn_bundle(TextBundle {
            style: Style {
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
                        value: "START".into(),
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
        .insert(TypingTarget::new_whole(
            "start".into(),
            vec![crate::Action::Start],
        ))
        .id();

    commands.entity(container).push_children(&[bg]);
    commands.entity(bg).push_children(&[starttext, starttarget]);
}

fn death_screen(
    mut commands: Commands,
    gltf_assets: Res<GltfAssets>,
    font_assets: Res<FontAssets>,
) {
    // rival

    commands
        .spawn_bundle((
            Transform::from_xyz(-5.4, 2.3, 4.5)
                .with_scale(Vec3::splat(2.5))
                .with_rotation(Quat::from_euler(EulerRot::XYZ, 0.1, -0.6, -0.7)),
            GlobalTransform::default(),
            StartScreen,
        ))
        .with_children(|parent| {
            parent.spawn_scene(gltf_assets.birb_gold.clone());
        });

    // text

    let container = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(0.),
                    left: Val::Px(0.),
                    ..Default::default()
                },
                size: Size::new(Val::Percent(50.0), Val::Percent(70.0)),
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
                size: Size::new(Val::Percent(70.0), Val::Percent(40.0)),
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::ColumnReverse,
                padding: Rect::all(Val::Px(10.0)),
                ..Default::default()
            },
            color: Color::BLACK.into(),
            ..Default::default()
        })
        .id();

    let deadtext = commands
        .spawn_bundle(TextBundle {
            style: Style {
                ..Default::default()
            },
            text: Text {
                sections: vec![TextSection {
                    value: "Oh wow, ouch!\nAre you alright?\nToo bad you're stuck at Z=0,\nthe path is a bit clearer a few units over.".into(),
                    style: TextStyle {
                        font: font_assets.main.clone(),
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

            text.sections[0].value = parts.0.to_uppercase();
            text.sections[1].value = parts.1.to_uppercase();
        }
    }
}

fn setup(
    mut commands: Commands,
    mut wordlist: ResMut<crate::typing::WordList>,
    font_assets: Res<FontAssets>,
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

    let mut not: HashSet<char> = "start".chars().collect();
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
                            font: font_assets.main.clone(),
                            font_size: 40.,
                            color: Color::GREEN,
                        },
                    },
                    TextSection {
                        value: topword.clone(),
                        style: TextStyle {
                            font: font_assets.main.clone(),
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
            vec![crate::Action::BirbUp, crate::Action::IncScore(1)],
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
                            font: font_assets.main.clone(),
                            font_size: 40.,
                            color: Color::GREEN,
                        },
                    },
                    TextSection {
                        value: bottomword.clone(),
                        style: TextStyle {
                            font: font_assets.main.clone(),
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
            vec![crate::Action::BirbDown, crate::Action::IncScore(1)],
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
                        font: font_assets.main.clone(),
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
