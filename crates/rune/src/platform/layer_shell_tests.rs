use super::{Layer, layer_for};

#[test]
fn top_layer_choice_is_independent_of_reservation() {
    assert!(matches!(layer_for(None, false), Layer::Overlay));
    assert!(matches!(layer_for(Some(40), false), Layer::Top));
    assert!(matches!(layer_for(None, true), Layer::Top));
    assert!(matches!(layer_for(Some(40), true), Layer::Top));
}
