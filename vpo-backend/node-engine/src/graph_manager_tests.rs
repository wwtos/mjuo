use crate::graph_manager::GraphManager;

#[test]
fn create_graph() {
    let mut graph_manager = GraphManager::new();

    let (index, _) = graph_manager.new_graph().unwrap();

    let graph = graph_manager.get_graph(index);
    assert!(graph.is_ok(), "Graph was none!");
}
