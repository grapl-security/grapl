// use proptest::proptest;
// use strategies::graph as g_strats;
//
// proptest! {
//     #[test]
//     fn test_merge_immutable_i64(
//         mut first in g_strats::immutable_uint_props(),
//         second in g_strats::immutable_uint_props(),
//     ) {
//         let original_first = first.clone();
//         first.merge_property(second);
//         assert_eq!(first, original_first);
//     }
//
// }
