use crate::{
    typing::{TypingTarget, WordList},
    util::cleanup,
    Action, AppState, FontAssets, GltfAssets, Score,
};
use bevy::{pbr::NotShadowCaster, prelude::*, scene::SceneInstance, utils::HashSet};

pub struct UiPlugin;

#[derive(Component)]
struct ScoreText;
#[derive(Component)]
struct StartScreenOnly;
#[derive(Component)]
struct EndScreenOnly;
#[derive(Component)]
struct RivalPortrait;
#[derive(Component)]
struct Decorated;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_targets);
        app.add_systems(Update, update_score);
        // Must run after SpawnScene schedule.
        app.add_systems(PostUpdate, decorate_rival_portrait);

        app.add_systems(OnExit(AppState::LoadingAssets), setup);

        app.add_systems(OnEnter(AppState::StartScreen), start_screen);
        app.add_systems(OnExit(AppState::StartScreen), cleanup::<StartScreenOnly>);

        app.add_systems(OnEnter(AppState::EndScreen), end_sceen);
        app.add_systems(OnExit(AppState::EndScreen), cleanup::<EndScreenOnly>);
    }
}

fn start_screen(
    mut commands: Commands,
    gltf_assets: Res<GltfAssets>,
    font_assets: Res<FontAssets>,
) {
    // rival

    commands.spawn((
        SceneBundle {
            scene: gltf_assets.birb_gold.clone(),
            transform: Transform::from_xyz(8.4, 4.0, -0.2)
                .with_scale(Vec3::splat(2.5))
                .with_rotation(Quat::from_euler(EulerRot::XYZ, -0.1, -2.5, -0.8)),
            ..default()
        },
        RivalPortrait,
        StartScreenOnly,
    ));

    // text

    let container = commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.),
                    right: Val::Px(0.),
                    width: Val::Percent(50.0),
                    height: Val::Percent(70.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                ..Default::default()
            },
            StartScreenOnly,
        ))
        .id();

    let bg = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(70.0),
                height: Val::Percent(40.0),
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::SpaceBetween,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                ..Default::default()
            },
            background_color: Color::BLACK.into(),
            ..Default::default()
        })
        .id();

    let starttext = commands
        .spawn(TextBundle {
            style: Style {
                ..Default::default()
            },
            text: Text {
                sections: vec![TextSection {
                    value: "So you want to join the flock, eh?\nYou'll have to beat me first!\nType the word below when you're ready."
                        .into(),
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

    let starttarget = commands
        .spawn((
            TextBundle {
                style: Style {
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
                            value: "START".into(),
                            style: TextStyle {
                                font: font_assets.main.clone(),
                                font_size: 40.,
                                color: Color::rgb_u8(255, 235, 146),
                            },
                        },
                    ],
                    ..Default::default()
                },
                ..Default::default()
            },
            TypingTarget::new_whole("start".into(), vec![Action::Start]),
        ))
        .id();

    commands.entity(container).push_children(&[bg]);
    commands.entity(bg).push_children(&[starttext, starttarget]);
}

fn end_sceen(
    mut commands: Commands,
    gltf_assets: Res<GltfAssets>,
    font_assets: Res<FontAssets>,
    score: Res<Score>,
) {
    let death_msg = if score.0 > 1000 {
        "I... wha... wow!\nWhat am I even doing with my life?\nThe flock is yours, if you'll have us!"
    } else if score.0 > 400 {
        "That was a close one!\nWith moves like that, you'll\nfit in well here!"
    } else if score.0 > 200 {
        "Not bad, kid!\nThere may be room for you in the flock\nas an unpaid apprentice."
    } else {
        "Oh wow, ouch!\nIt's a shame you can't move left or right,\nthe path is a bit clearer over here!"
    };

    // rival

    commands.spawn((
        SceneBundle {
            scene: gltf_assets.birb_gold.clone(),
            transform: Transform::from_xyz(8.4, 4.0, -0.2)
                .with_scale(Vec3::splat(2.5))
                .with_rotation(Quat::from_euler(EulerRot::XYZ, -0.1, -2.5, -0.8)),
            ..default()
        },
        RivalPortrait,
        EndScreenOnly,
    ));

    // text

    let container = commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.),
                    right: Val::Px(0.),
                    width: Val::Percent(50.0),
                    height: Val::Percent(70.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                ..Default::default()
            },
            EndScreenOnly,
        ))
        .id();

    let bg = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(70.0),
                height: Val::Percent(40.0),
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::SpaceBetween,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                ..Default::default()
            },
            background_color: Color::BLACK.into(),
            ..Default::default()
        })
        .id();

    let deadtext = commands
        .spawn(TextBundle {
            style: Style {
                ..Default::default()
            },
            text: Text {
                sections: vec![TextSection {
                    value: death_msg.into(),
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

    let retrytext = commands
        .spawn((
            TextBundle {
                style: Style {
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
                            value: "RETRY".into(),
                            style: TextStyle {
                                font: font_assets.main.clone(),
                                font_size: 40.,
                                color: Color::rgb_u8(255, 235, 146),
                            },
                        },
                    ],
                    ..Default::default()
                },
                ..Default::default()
            },
            TypingTarget::new_whole("retry".into(), vec![Action::Retry]),
        ))
        .id();

    commands.entity(container).push_children(&[bg]);
    commands.entity(bg).push_children(&[deadtext, retrytext]);
}

fn update_score(mut query: Query<&mut Text, With<ScoreText>>, score: Res<Score>) {
    if !score.is_changed() {
        return;
    }
    for mut text in query.iter_mut() {
        text.sections[1].value = format!("{}", score.0);
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

fn setup(mut commands: Commands, mut wordlist: ResMut<WordList>, font_assets: Res<FontAssets>) {
    // root node
    let root = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            ..Default::default()
        })
        .id();

    let topbar = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(50.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect {
                    bottom: Val::Px(5.),
                    ..Default::default()
                },
                ..Default::default()
            },
            background_color: Color::BLACK.into(),
            ..Default::default()
        })
        .id();

    let mut not: HashSet<char> = "start".chars().collect();
    let topword = wordlist.find_next_word(&not);
    for c in topword.chars() {
        not.insert(c);
    }

    let toptext = commands
        .spawn((
            TextBundle {
                style: Style {
                    margin: UiRect::all(Val::Px(5.0)),
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
                                color: Color::rgb_u8(255, 235, 146),
                            },
                        },
                    ],
                    ..Default::default()
                },
                ..Default::default()
            },
            TypingTarget::new(topword, vec![Action::BirbUp, Action::IncScore(1)]),
        ))
        .id();

    let bottombar = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(50.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect {
                    bottom: Val::Px(5.),
                    ..Default::default()
                },
                ..Default::default()
            },
            background_color: Color::BLACK.into(),
            ..Default::default()
        })
        .id();

    let bottomword = wordlist.find_next_word(&not);
    let bottomtext = commands
        .spawn((
            TextBundle {
                style: Style {
                    margin: UiRect::all(Val::Px(5.0)),
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
                                color: Color::rgb_u8(255, 235, 146),
                            },
                        },
                    ],
                    ..Default::default()
                },
                ..Default::default()
            },
            TypingTarget::new(bottomword, vec![Action::BirbDown, Action::IncScore(1)]),
        ))
        .id();

    let scoretext = commands
        .spawn((
            TextBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(3.0),
                    left: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(5.0)),
                    ..Default::default()
                },
                text: Text {
                    sections: vec![
                        TextSection {
                            value: "SCORE ".into(),
                            style: TextStyle {
                                font: font_assets.main.clone(),
                                font_size: 40.,
                                color: Color::rgba(0.8, 0.8, 0.8, 1.0),
                            },
                        },
                        TextSection {
                            value: "0".into(),
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
            },
            ScoreText,
        ))
        .id();

    commands.entity(root).push_children(&[topbar, bottombar]);
    commands.entity(topbar).push_children(&[toptext, scoretext]);
    commands.entity(bottombar).push_children(&[bottomtext]);
}

fn decorate_rival_portrait(
    mut commands: Commands,
    spawner: Res<SceneSpawner>,
    undecorated: Query<(Entity, &SceneInstance), (Without<Decorated>, With<RivalPortrait>)>,
    mesh_query: Query<(), With<Handle<Mesh>>>,
) {
    for (entity, instance) in undecorated.iter() {
        if spawner.instance_is_ready(**instance) {
            for instance_entity in spawner.iter_instance_entities(**instance) {
                if mesh_query.get(instance_entity).is_ok() {
                    commands.entity(instance_entity).insert(NotShadowCaster);
                }
            }

            commands.entity(entity).insert(Decorated);
        }
    }
}
