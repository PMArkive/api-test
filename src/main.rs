mod harness;
mod report;

use crate::harness::Harness;
use bitbuffer::{BitReadBuffer, LittleEndian};
use color_eyre::{eyre::WrapErr, Report, Result};
use demostf_client::{ChatMessage, Class, Demo, ListParams, SteamID, Team};
use report::{assert_eq, Test};
use std::str::FromStr;
use tf_demo_parser::{demo::header::Header, DemoParser, MatchState};

macro_rules! assert_object_eq {
    ($obj:expr => { $($name:ident == $value:expr),* }) => {
        $(report::assert_eq_borrow(&$obj.$name, $value)?;)*
    };
    ($obj:expr => { $($name:ident == $value:expr),* , }) => {
        $(report::assert_eq_borrow(&$obj.$name, $value)?;)*
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    let harness = Harness::new(&dotenv::var("BASE_URL")?, &dotenv::var("DB_URL")?).await?;
    let granary_data = include_bytes!("../data/granary.dem");
    let process_data = include_bytes!("../data/process.dem");
    let warmfrost_data = include_bytes!("../data/warmfrost.dem");
    let coalplant_data = include_bytes!("../data/coalplant.dem");

    Test::run(
        "Upload with invalid credentials",
        &harness,
        |test| async move {
            test.step("upload", |client| async move {
                let result = client
                    .upload_demo(
                        String::from("test.dem"),
                        granary_data.to_vec(),
                        String::from("RED"),
                        String::from("BLUE"),
                        String::from("wrong_token"),
                    )
                    .await;

                match result {
                    Ok(_) => Err(Report::msg("Expected error during upload")),
                    Err(demostf_client::Error::InvalidApiKey) => Ok(()),
                    Err(e) => Err(Report::msg(format!(
                        "Unexpected error during upload: {}",
                        e
                    ))),
                }
            })
            .await?;
            Ok(())
        },
    )
    .await;

    Test::run(
        "Upload demo, then retrieve info",
        &harness,
        |test| async move {
            let parser =
                DemoParser::new(BitReadBuffer::new(granary_data.to_vec(), LittleEndian).into());
            let (header, state) = parser
                .parse()
                .map_err(|_| Report::msg("Failed to parse demo"))?;
            let state = &state;

            let id = test
                .step("upload", |client| async move {
                    Ok(client
                        .upload_demo(
                            String::from("test.dem"),
                            granary_data.to_vec(),
                            String::from("RED"),
                            String::from("BLUE"),
                            String::from("token"),
                        )
                        .await?)
                })
                .await?;

            assert_eq(id, 1)?;

            test.step("get demo", |client| async move {
                let demo = client.get(id).await?;
                assert_object_eq!(demo => {
                    id == 1,
                    name == "test.dem",
                    map == "cp_granary_pro_rc8",
                    red_score == 0,
                    blue_score == 1,
                    player_count == 12,
                });
                verify_demo(&demo, &header, state)?;
                assert_eq(demo.uploader.id(), 1)?;

                let uploader = demo.uploader.resolve(client).await?;
                assert_eq(&uploader.name, "Icewind")?;

                Ok(())
            })
            .await?;

            test.step("list demos", |client| async move {
                let list = client.list(ListParams::default(), 1).await?;
                assert_eq(list.len(), 1)?;
                assert_object_eq!(list[0] => {
                    id == 1,
                    name == "test.dem",
                    map == "cp_granary_pro_rc8",
                    red_score == 0,
                    blue_score == 1,
                    player_count == 12,
                });
                assert_eq(list[0].uploader.id(), 1)?;

                let page2 = client.list(ListParams::default(), 2).await?;
                assert_eq(page2.len(), 0)?;

                Ok(())
            })
            .await?;

            test.step("chat", |client| async move {
                let chat = client.get_chat(id).await?;
                verify_chat(&chat, state)
            })
            .await?;

            test.step("upload_again", |client| async move {
                let new_id = client
                    .upload_demo(
                        String::from("test.dem"),
                        granary_data.to_vec(),
                        String::from("RED"),
                        String::from("BLUE"),
                        String::from("token"),
                    )
                    .await?;

                assert_eq(id, new_id)?;
                Ok(())
            })
            .await?;

            Ok(())
        },
    )
    .await;

    Test::run("Listings", &harness, |test| async move {
        test.step("upload", |client| async move {
            client
                .upload_demo(
                    String::from("test1.dem"),
                    granary_data.to_vec(),
                    String::from("RED"),
                    String::from("BLUE"),
                    String::from("token"),
                )
                .await?;
            client
                .upload_demo(
                    String::from("test2.dem"),
                    process_data.to_vec(),
                    String::from("RED"),
                    String::from("BLUE"),
                    String::from("token"),
                )
                .await?;
            client
                .upload_demo(
                    String::from("test3.dem"),
                    warmfrost_data.to_vec(),
                    String::from("RED"),
                    String::from("BLUE"),
                    String::from("token"),
                )
                .await?;
            client
                .upload_demo(
                    String::from("test3.dem"),
                    coalplant_data.to_vec(),
                    String::from("RED"),
                    String::from("BLUE"),
                    String::from("token"),
                )
                .await?;
            Ok(())
        })
        .await?;

        test.step("list defaults", |client| async move {
            let list = client.list(ListParams::default(), 1).await?;
            assert_eq(list.len(), 4)?;
            assert_eq(list[0].id, 4)?;
            assert_eq(list[1].id, 3)?;
            assert_eq(list[2].id, 2)?;
            assert_eq(list[3].id, 1)?;
            Ok(())
        })
        .await?;

        Ok(())
    })
    .await;

    Ok(())
}

fn verify_demo(api_result: &Demo, header: &Header, state: &MatchState) -> Result<()> {
    use tf_demo_parser::demo::parser::gamestateanalyser;

    fn map_team(team: Team) -> gamestateanalyser::Team {
        match team {
            Team::Red => gamestateanalyser::Team::Red,
            Team::Blue => gamestateanalyser::Team::Blue,
        }
    }

    fn map_class(class: Class) -> gamestateanalyser::Class {
        match class {
            Class::Scout => gamestateanalyser::Class::Scout,
            Class::Soldier => gamestateanalyser::Class::Soldier,
            Class::Pyro => gamestateanalyser::Class::Pyro,
            Class::Demoman => gamestateanalyser::Class::Demoman,
            Class::HeavyWeapons => gamestateanalyser::Class::Heavy,
            Class::Medic => gamestateanalyser::Class::Medic,
            Class::Engineer => gamestateanalyser::Class::Engineer,
            Class::Sniper => gamestateanalyser::Class::Sniper,
            Class::Spy => gamestateanalyser::Class::Spy,
        }
    }

    assert_eq(&api_result.map, &header.map).wrap_err("Failed to compare map")?;
    assert_eq(
        api_result.red_score,
        state
            .rounds
            .iter()
            .filter(|round| round.winner == gamestateanalyser::Team::Red)
            .count() as u8,
    )
    .wrap_err("Failed to compare red score")?;
    assert_eq(
        api_result.blue_score,
        state
            .rounds
            .iter()
            .filter(|round| round.winner == gamestateanalyser::Team::Blue)
            .count() as u8,
    )
    .wrap_err("Failed to compare blue score")?;
    assert_eq(&api_result.server, &header.server).wrap_err("Failed to compare server")?;
    assert_eq(&api_result.nick, &header.nick).wrap_err("Failed to compare server")?;
    assert_eq(api_result.duration, header.duration as u16).wrap_err("Failed to compare server")?;

    let mut players = state
        .users
        .values()
        .filter(|user| user.team.is_player())
        .collect::<Vec<_>>();

    players.sort_by(|a, b| {
        SteamID::from_str(&a.steam_id)
            .unwrap()
            .account_id()
            .cmp(&SteamID::from_str(&b.steam_id).unwrap().account_id())
    });

    let mut api_players = api_result.players.iter().collect::<Vec<_>>();
    api_players.sort_by(|a, b| {
        a.user
            .steam_id
            .account_id()
            .cmp(&b.user.steam_id.account_id())
    });

    assert_eq(api_result.player_count, players.len() as u8)
        .wrap_err("Failed to compare player count")?;
    assert_eq(api_players.len(), players.len()).wrap_err("Failed to compare player count")?;

    for (api_player, player) in api_players.iter().zip(players.iter()) {
        assert_eq(&api_player.user.name, &player.name).wrap_err_with(|| {
            format!("Failed to compare player name for {}", api_player.user.name)
        })?;
        assert_eq(
            &api_player.user.steam_id,
            &SteamID::from_str(&player.steam_id).unwrap(),
        )
        .wrap_err_with(|| format!("Failed to compare steam id for {}", api_player.user.name))?;
        assert_eq(map_team(api_player.team), player.team)
            .wrap_err_with(|| format!("Failed to compare team for {}", api_player.user.name))?;
        assert_eq(
            map_class(api_player.class),
            player.classes.sorted().next().unwrap().0,
        )
        .wrap_err_with(|| format!("Failed to compare class for {}", api_player.user.name))?;
        let kills = state
            .deaths
            .iter()
            .filter(|kill| kill.killer == player.user_id)
            .count() as u8;
        let assists = state
            .deaths
            .iter()
            .filter(|kill| kill.assister == Some(player.user_id))
            .count() as u8;
        let deaths = state
            .deaths
            .iter()
            .filter(|kill| kill.victim == player.user_id)
            .count() as u8;
        assert_eq(api_player.kills, kills)
            .wrap_err_with(|| format!("Failed to compare kills for {}", api_player.user.name))?;
        assert_eq(api_player.assists, assists)
            .wrap_err_with(|| format!("Failed to compare assists for {}", api_player.user.name))?;
        assert_eq(api_player.deaths, deaths)
            .wrap_err_with(|| format!("Failed to compare deaths for {}", api_player.user.name))?;
    }

    Ok(())
}

fn verify_chat(chat: &[ChatMessage], state: &MatchState) -> Result<()> {
    assert_eq(chat.len(), state.chat.len())
        .wrap_err("Failed to compare number of chat messages")?;

    let mut demo_chat = state.chat.clone();
    demo_chat.sort_by(|a, b| a.tick.cmp(&b.tick));

    for (api_chat, chat) in chat.iter().zip(demo_chat.iter()) {
        assert_eq(&api_chat.message, &chat.text).wrap_err("Failed to compare chat message")?;
        assert_eq(&api_chat.user, &chat.from).wrap_err("Failed to compare chat message sender")?;
        // assert_eq(
        //     api_chat.time,
        //     (chat.tick as f32 * state.interval_per_tick) as u32,
        // )
        // .wrap_err("Failed to compare chat message time")?;
    }

    Ok(())
}
