use bonsai_bt::{
    Behavior::{Action, Sequence, Wait, WaitForever, WhenAny, While},
    Event, Running, State, Success, Timer, UpdateArgs, BT, RUNNING,
};
use std::{collections::HashMap, thread::sleep, time::Duration};

type Damage = u32;
type Distance = Vec<f64>;

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq)]
enum EnemyNPC {
    Run,
    Attack,
    // // When player is close -> melee attack
    // // distance [m], damage
    // MeleeAttack(Distance, Damage),
    // When player is far away -> fire weapon
    FireWeapon(Distance),
}

#[cfg(test)]
mod tests {
    use super::*;

     #[test]
    fn wait_tick_test() {
        let game_tick = |dt: f64, state: &mut State<EnemyNPC>| -> usize {
            let mut attack_count = 0;
            let e: Event = UpdateArgs { dt }.into(); // 행동 트리를 얼마나 앞으로 이동
            state
                .tick(&e, &mut |args: bonsai_bt::ActionArgs<Event, EnemyNPC>| {
                    println!("SEQUENCE , remain delta time : {}", args.dt);
                    match *args.action {
                        EnemyNPC::Run => {
                            println!("-> 1. Run");
                            (bonsai_bt::Success, args.dt)
                        }
                        EnemyNPC::Attack => {
                            println!("-> 2. Attack");
                            attack_count += 1;
                            (bonsai_bt::Success, args.dt)
                        }
                        _ => RUNNING,
                    }
                });
            attack_count
        };

        // run과 attack은 delta time이 0인 작업이다
        // 하지만 같은 sequence에 있는 Wait에 의해 10.0초(delta time)가 걸린다
        let sequence = Sequence(vec![
            Action(EnemyNPC::Run),
            Action(EnemyNPC::Attack),
            Wait(10.0),
        ]);
        let mut state =
            State::new(
                While(
                    Box::new(WaitForever),
                    vec![
                        sequence,
                    ]
                )
            );

        //==> progress 3
        assert_eq!(game_tick(3.0, &mut state), 1);

        //==> progress 6
        assert_eq!(game_tick(3.0, &mut state), 0);

        //==> progress 9
        assert_eq!(game_tick(3.0, &mut state), 0);

        // progress 12 = (10 + 2)
        //     10 : 1 sequence
        //     2 : remain delta. so progress 2.
        assert_eq!(game_tick(3.0, &mut state), 1);

        // progress 10 = (2 + 8)
        assert_eq!(game_tick(8.0, &mut state), 1);
    }

    #[test]
    fn dynamic_wait_tick_test() {
        let game_tick = |mut run_count: usize,
                         dt: f64,
                         dt_timer: &mut f64,
                         weapon_idx: usize,
                         state: &mut State<EnemyNPC>|
                         -> usize {
            let dt = dt + *dt_timer;
            let e: Event = UpdateArgs { dt }.into();
            state.tick(&e, &mut |args| match &*args.action {
                EnemyNPC::Run => {
                    run_count += 1;
                    (Success, args.dt)
                }
                EnemyNPC::FireWeapon(times) => {
                    let wait_t = times[weapon_idx.to_owned()];
                    if args.dt >= wait_t {
                        *dt_timer = args.dt - wait_t;
                        (Success, *dt_timer)
                    } else {
                        *dt_timer = args.dt;
                        (Running, *dt_timer)
                    }
                }
                _ => RUNNING,
            });
            run_count
        };

        let mut state = State::new(While(
            Box::new(Wait(50.0)),
            // first time wait 1.0s, then 2.0s, then 3.0s
            vec![
                Action(EnemyNPC::FireWeapon(vec![1.0, 2.0, 3.0])),
                Action(EnemyNPC::Run),
            ],
        ));

        let mut run_count: usize = 0;
        let mut dt_timer: f64 = 0.0;

        // progress 0, wait time 1
        run_count = game_tick(run_count, 1.0, &mut dt_timer, 0, &mut state);
        assert_eq!(run_count, 1);
        assert_eq!(dt_timer, 0.0);

        // progress 1.5, wait time 2
        run_count = game_tick(run_count, 1.5, &mut dt_timer, 1, &mut state);
        assert_eq!(run_count, 1);
        assert_eq!(dt_timer, 1.5);

        // progress 0.5 = (1.5 + 3.0) - 2 - 2
        //                             run run
        run_count = game_tick(run_count, 3.0, &mut dt_timer, 1, &mut state);
        assert_eq!(run_count, 3);
        assert_eq!(dt_timer, 0.5);
    }

    #[test]
    fn while_wait_test() {
        let mut state = State::new(
            While(
                Box::new(Wait(0.5)),
                vec![
                    Action(EnemyNPC::Run),
                ]
            )
        );
        let mut flag: bool = false;
        let dt = 0.0;
        let e: Event = UpdateArgs { dt }.into();
        state.tick(&e, &mut |args| {
            match args.action {
                EnemyNPC::Run => {
                    sleep(Duration::from_secs(1));
                    flag = true;
                    (bonsai_bt::Failure, args.dt) // to break
                }
                _ => RUNNING,
            }
        });
        assert!(flag);
    }
    #[test]
    fn black_board_test() {
        let game_tick = |dt: f64, bt: &mut BT<EnemyNPC, String, f64>| -> f64 {
            let mut test_data_res: f64 = 0.0;
            let e: Event = UpdateArgs { dt }.into();
            let db = &*bt.get_blackboard().get_db();
            let test_data_1: f64 = *db.get("test_data_1").unwrap();
            let test_data_2: f64 = *db.get("test_data_2").unwrap();
            bt.state.tick(&e, &mut |args| {
                match args.action {
                    EnemyNPC::Run => {
                        test_data_res = test_data_1 + test_data_2;
                        (bonsai_bt::Success, args.dt) // to break
                    }
                    _ => RUNNING,
                }
            });
            test_data_res
        };
        let mut blackboard: HashMap<String, f64> = HashMap::new();
        blackboard.insert("test_data_1".to_string(), 10.0);
        blackboard.insert("test_data_2".to_string(), 20.0);
        assert_eq!(
            game_tick(0.0, &mut BT::new(Action(EnemyNPC::Run), blackboard)), 30.0) ;
    }
}
