use crate::{model::*, solver::*};

fn roster() -> Vec<Attendee> {
    serde_json::from_str(
        r#"[
        {"id":"a","name":"A","availability":"Monday"},
        {"id":"b","name":"B","availability":"Monday"}
    ]"#,
    )
    .unwrap()
}
fn availability(a: &[(Day, i64, i64)], b: &[(Day, i64, i64)]) -> ParsedAvailability {
    ParsedAvailability {
        attendees: [("a", a), ("b", b)]
            .into_iter()
            .map(|(id, windows)| AttendeeAvailability {
                attendee_id: id.into(),
                windows: windows
                    .iter()
                    .map(|&(day, start_hour, end_hour)| TimeWindow {
                        day,
                        start_hour,
                        end_hour,
                    })
                    .collect(),
            })
            .collect(),
    }
}

#[test]
fn unique_solution_and_full_hour_boundaries() {
    let data = validate_availability(
        &roster(),
        availability(&[(Day::Wednesday, 10, 12)], &[(Day::Wednesday, 11, 14)]),
    )
    .unwrap();
    assert_eq!(
        solve(&data).unwrap(),
        Schedule::Scheduled(Slot {
            day: Day::Wednesday,
            start_hour: 11
        })
    );
    assert!(!data.permits(Slot {
        day: Day::Wednesday,
        start_hour: 12
    }));
    for (start, end, expected) in [
        (8, 9, None),
        (8, 10, Some(9)),
        (16, 17, Some(16)),
        (17, 24, None),
    ] {
        let data = validate_availability(
            &roster(),
            availability(&[(Day::Monday, start, end)], &[(Day::Monday, 0, 24)]),
        )
        .unwrap();
        let result = solve(&data).unwrap();
        assert_eq!(
            result,
            expected.map_or(Schedule::NoSlot, |start_hour| Schedule::Scheduled(Slot {
                day: Day::Monday,
                start_hour
            }))
        );
    }
}

#[test]
fn real_solver_agrees_with_independent_finite_search() {
    // Includes empty availability, disjoint days/hours, multiple windows/solutions,
    // overlapping intervals, and intervals outside business hours.
    let cases = [
        vec![],
        vec![(Day::Monday, 9, 10)],
        vec![(Day::Monday, 10, 12)],
        vec![(Day::Tuesday, 9, 17)],
        vec![(Day::Monday, 0, 24)],
        vec![(Day::Friday, 17, 24)],
        vec![(Day::Monday, 9, 11), (Day::Tuesday, 11, 14)],
        vec![(Day::Monday, 9, 12), (Day::Monday, 10, 13)],
    ];
    for a in &cases {
        for b in &cases {
            let data = validate_availability(&roster(), availability(a, b)).unwrap();
            let slots: Vec<_> = Day::ALL
                .into_iter()
                .flat_map(|day| (9..17).map(move |start_hour| Slot { day, start_hour }))
                .filter(|slot| data.permits(*slot))
                .collect();
            match solve(&data).unwrap() {
                Schedule::Scheduled(slot) => assert!(slots.contains(&slot)),
                Schedule::NoSlot => assert!(slots.is_empty()),
                Schedule::Inconclusive(reason) => {
                    panic!("tiny deterministic fixture was inconclusive: {reason}")
                }
            }
        }
    }
}

#[test]
fn rejects_roster_mismatch_and_invalid_hours() {
    for id in ["unknown", "a"] {
        let mut parsed = availability(&[], &[]);
        parsed.attendees[1].attendee_id = id.into();
        assert!(validate_availability(&roster(), parsed).is_err());
    }
    let mut parsed = availability(&[], &[]);
    parsed.attendees.pop();
    assert!(validate_availability(&roster(), parsed).is_err());
    for (start, end) in [
        (-1, 10),
        (0, 25),
        (10, 10),
        (11, 10),
        (24, 24),
        (0, 0),
        (i64::MAX, i64::MAX),
    ] {
        assert!(
            validate_availability(&roster(), availability(&[(Day::Monday, start, end)], &[]))
                .is_err()
        );
    }
    assert!(validate_roster(&[]).is_err());
    let mut duplicated = roster();
    duplicated[1].id = "a".into();
    assert!(validate_roster(&duplicated).is_err());
    for field in ["id", "name", "availability"] {
        let mut empty = serde_json::to_value(roster()).unwrap();
        empty[0][field] = " ".into();
        assert!(validate_roster(&serde_json::from_value::<Vec<Attendee>>(empty).unwrap()).is_err());
    }
}

#[test]
fn extraction_display_uses_original_names_and_shows_empty_windows() {
    let display = format_extraction(&roster(), &availability(&[], &[]));
    assert!(display.contains("A [a]: no availability"));
    assert!(display.contains("B [b]: no availability"));
}
