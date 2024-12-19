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
                ..default()
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
                ..default()
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
                ..default()
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
                ..default()
            },
            BackgroundColor(Color::BLACK.into()),
        ))
        .id();

    let deadtext = commands
        .spawn((
            Text::new(death_msg),
            TextFont {
                font: font_assets.main.clone(),
                font_size: 40.,
                ..default()
            },
            TextColor(Color::WHITE),
        ))
        .id();

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
        text.0 = format!("{}", score.0);
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
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .id();

    let topbar = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(50.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect {
                    bottom: Val::Px(5.),
                    ..default()
                },
                ..default()
            },
            BackgroundColor(Color::BLACK.into()),
        ))
        .id();

    let mut not: HashSet<char> = "start".chars().collect();
    let topword = wordlist.find_next_word(&not);
    for c in topword.chars() {
        not.insert(c);
    }

    let toptext = commands
        .spawn((
            Text::default(),
            TextFont {
                font: font_assets.main.clone(),
                font_size: 40.,
                ..default()
            },
            TextColor(LIME.into()),
            Node {
                margin: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            TypingTarget::new(topword.clone(), vec![Action::BirbUp, Action::IncScore(1)]),
        ))
        .with_child((
            TextSpan::new(topword),
            TextFont {
                font: font_assets.main.clone(),
                font_size: 40.,
                ..default()
            },
            TextColor(Color::srgb_u8(255, 235, 146)),
        ))
        .id();

    let bottombar = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(50.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect {
                    bottom: Val::Px(5.),
                    ..default()
                },
                ..default()
            },
            BackgroundColor(Color::BLACK.into()),
        ))
        .id();

    let bottomword = wordlist.find_next_word(&not);
    let bottomtext = commands
        .spawn((
            Text::default(),
            TextFont {
                font: font_assets.main.clone(),
                font_size: 40.,
                ..default()
            },
            TextColor(LIME.into()),
            Node {
                margin: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            TypingTarget::new(
                bottomword.clone(),
                vec![Action::BirbDown, Action::IncScore(1)],
            ),
        ))
        .with_child((
            TextSpan::new(bottomword),
            TextFont {
                font: font_assets.main.clone(),
                font_size: 40.,
                ..default()
            },
            TextColor(Color::srgb_u8(255, 235, 146)),
        ))
        .id();

    let scoretext = commands
        .spawn((
            Text::new("SCORE "),
            TextFont {
                font: font_assets.main.clone(),
                font_size: 40.,
                ..default()
            },
            TextColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(3.0),
                left: Val::Px(10.0),
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            ScoreText,
        ))
        .with_child((
            TextSpan::new("0"),
            TextFont {
                font: font_assets.main.clone(),
                font_size: 40.,
                ..default()
            },
            TextColor(Color::WHITE),
        ))
        .id();

    commands.entity(root).add_children(&[topbar, bottombar]);
    commands.entity(topbar).add_children(&[toptext, scoretext]);
    commands.entity(bottombar).add_children(&[bottomtext]);
}

fn decorate_rival_portrait(
    mut commands: Commands,
    spawner: Res<SceneSpawner>,
    undecorated: Query<(Entity, &SceneInstance), (Without<Decorated>, With<RivalPortrait>)>,
    mesh_query: Query<(), With<Mesh3d>>,
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
