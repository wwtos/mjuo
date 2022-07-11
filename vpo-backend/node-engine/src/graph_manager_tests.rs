use crate::graph_manager::GraphManager;

#[test]
fn create_graph() {
    let mut graph_manager = GraphManager::default();

    let index = graph_manager.new_graph();
    assert_eq!(index, 0);

    let graph = graph_manager.get_graph_ref(index);
    assert!(graph.is_some(), "Graph was none!");
}
