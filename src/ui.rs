use crate::{
    typing::{TypingTarget, WordList},
    util::cleanup,
    Action, AppState, FontAssets, GltfAssets, Score,
};
use bevy::{
    color::palettes::css::LIME, pbr::NotShadowCaster, prelude::*, scene::SceneInstance,
    utils::HashSet,
};

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
        SceneRoot(gltf_assets.birb_gold.clone()),
        Transform::from_xyz(8.4, 4.0, -0.2)
            .with_scale(Vec3::splat(2.5))
            .with_rotation(Quat::from_euler(EulerRot::XYZ, -0.1, -2.5, -0.8)),
        RivalPortrait,
        StartScreenOnly,
    ));

    // text

    let container = commands
        .spawn((
            Node {
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
            StartScreenOnly,
        ))
        .id();

    let bg = commands
        .spawn((
            Node {
                width: Val::Percent(70.0),
                height: Val::Percent(40.0),
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::SpaceBetween,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                ..Default::default()
            },
            BackgroundColor(Color::BLACK.into()),
        ))
        .id();

    let starttext = commands
        .spawn((
            Text::new(concat!(
                "So you want to join the flock, eh?\n",
                "You'll have to beat me first!\n",
                "Type the word below when you're ready."
            )),
            TextFont {
                font: font_assets.main.clone(),
                font_size: 40.,
                ..default()
            },
            TextColor(Color::WHITE),
        ))
        .id();

    let starttarget = commands
        .spawn((
            Text::default(),
            TextFont {
                font: font_assets.main.clone(),
                font_size: 40.,
                ..default()
            },
            TextColor(LIME.into()),
            TypingTarget::new_whole("start".into(), vec![Action::Start]),
        ))
        .with_child((
            TextSpan::new("START"),
            TextFont {
                font: font_assets.main.clone(),
                font_size: 40.,
                ..default()
            },
            TextColor(Color::srgb_u8(255, 235, 146)),
        ))
        .id();

    commands.entity(container).add_children(&[bg]);
    commands.entity(bg).add_children(&[starttext, starttarget]);
}

fn end_sceen(
    mut commands: Commands,
    gltf_assets: Res<GltfAssets>,
    font_assets: Res<FontAssets>,
    score: Res<Score>,
) {
    let death_msg = if score.0 > 1000 {
        concat!(
            "I... wha... wow!\n,
            What am I even doing with my life?\n",
            "The flock is yours, if you'll have us!"
        )
    } else if score.0 > 400 {
        concat!(
            "That was a close one!\n",
            "With moves like that, you'll\n",
            "fit in well here!"
        )
    } else if score.0 > 200 {
        concat!(
            "Not bad, kid!\n",
            "There may be room for you in the flock\n",
            "as an unpaid apprentice."
        )
    } else {
        concat!(
            "Oh wow, ouch!\n",
            "It's a shame you can't move side to side,\n",
            "the path is a bit clearer over here!"
        )
    };

    // rival

    commands.spawn((
        SceneRoot(gltf_assets.birb_gold.clone()),
        Transform::from_xyz(8.4, 4.0, -0.2)
            .with_scale(Vec3::splat(2.5))
            .with_rotation(Quat::from_euler(EulerRot::XYZ, -0.1, -2.5, -0.8)),
        RivalPortrait,
        Name::new("RivalPortrait"),
        EndScreenOnly,
    ));

    // text

    let container = commands
        .spawn((
            Node {
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
            EndScreenOnly,
        ))
        .id();

    let bg = commands
        .spawn((
            Node {
                width: Val::Percent(70.0),
                height: Val::Percent(40.0),
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::SpaceBetween,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                ..Default::default()
            },
            BackgroundColor(Color::BLACK.into()),
        ))
        .id();

    let deadtext = commands.spawn((
        Text::new(death_msg),
        TextFont {
            font: font_assets.main.clone(),
            font_size: 40.,
            ..default()
        },
        TextColor(Color::WHITE),
    ));

    let retrytext = commands
        .spawn((
            Text::default(),
            TextFont {
                font: font_assets.main.clone(),
                font_size: 40.,
                ..default()
            },
            TextColor(LIME.into()),
            TypingTarget::new_whole("retry".into(), vec![Action::Retry]),
        ))
        .with_child((
            TextSpan::new("RETRY"),
            TextFont {
                font: font_assets.main.clone(),
                font_size: 40.,
                ..default()
            },
            TextColor(Color::srgb_u8(255, 235, 146)),
        ))
        .id();

    commands.entity(container).add_children(&[bg]);
    commands.entity(bg).add_children(&[deadtext, retrytext]);
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
    mut writer: TextUiWriter,
) {
    for (entity, target) in query.iter() {
        let parts = target.word.split_at(target.index);

        *writer.text(entity, 0) = parts.0.to_uppercase();
        *writer.text(entity, 1) = parts.1.to_uppercase();
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
                                color: LIME.into(),
                            },
                        },
                        TextSection {
                            value: topword.clone(),
                            style: TextStyle {
                                font: font_assets.main.clone(),
                                font_size: 40.,
                                color: Color::srgb_u8(255, 235, 146),
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
                                color: LIME.into(),
                            },
                        },
                        TextSection {
                            value: bottomword.clone(),
                            style: TextStyle {
                                font: font_assets.main.clone(),
                                font_size: 40.,
                                color: Color::srgb_u8(255, 235, 146),
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
                                color: Color::srgba(0.8, 0.8, 0.8, 1.0),
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
