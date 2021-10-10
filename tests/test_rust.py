import generative.rust as gr
import pytest


def test_dla_parameters():
    p = gr.Parameters(
        seeds=1,
        seed=2,
        particle_spacing=3,
        attraction_distance=4,
        min_move_distance=5,
        stubbornness=6,
        stickiness=7,
    )

    assert isinstance(p.seeds, int)
    assert p.seeds == 1
    p.seeds = 10
    assert p.seeds == 10

    assert isinstance(p.seed, int)
    assert p.seed == 2
    p.seed = 20
    assert p.seed == 20

    assert isinstance(p.particle_spacing, float)
    assert p.particle_spacing == 3
    p.particle_spacing = 30
    assert p.particle_spacing == 30

    assert isinstance(p.attraction_distance, float)
    assert p.attraction_distance == 4
    p.attraction_distance = 40
    assert p.attraction_distance == 40

    assert isinstance(p.min_move_distance, float)
    assert p.min_move_distance == 5
    p.min_move_distance = 50
    assert p.min_move_distance == 50

    assert isinstance(p.stubbornness, int)
    assert p.stubbornness == 6
    p.stubbornness = 60
    assert p.stubbornness == 60

    assert isinstance(p.stickiness, float)
    assert p.stickiness == 7
    p.stickiness = 70
    assert p.stickiness == 70


def test_dla_particle_default_new():
    p = gr.Particle()

    assert isinstance(p.x, float)
    assert isinstance(p.y, float)
    assert isinstance(p.coordinates, list)
    assert isinstance(p.join_attempts, int)

    assert p.x == 0
    assert p.y == 0
    assert p.join_attempts == 0

    p = gr.Particle(x=1)
    assert p.x == 1
    assert p.y == 0
    assert p.join_attempts == 0

    p = gr.Particle(1, 2)
    assert p.x == 1
    assert p.y == 2
    assert p.join_attempts == 0

    p = gr.Particle(1, 2, 3)
    assert p.x == 1
    assert p.y == 2
    assert p.join_attempts == 3

    p.join_attempts = 4
    assert p.join_attempts == 4

    with pytest.raises(TypeError):
        p.join_attempts = 4.0


def test_dla_particle_coordinates():
    p = gr.Particle()

    assert p.x == 0
    assert p.y == 0
    assert p.coordinates == [0, 0]

    p.x = 1
    assert p.x == 1
    assert p.coordinates == [1, 0]

    p.y = 2
    assert p.y == 2
    assert p.coordinates == [1, 2]

    p.coordinates = [3, 4]
    assert p.x == 3
    assert p.y == 4
    assert p.coordinates == [3, 4]

    with pytest.raises(ValueError):
        p.coordinates = []
    with pytest.raises(ValueError):
        p.coordinates = [1]
    with pytest.raises(ValueError):
        p.coordinates = [1, 2, 3]

    with pytest.raises(TypeError):
        p.coordinates = ["str"]
    with pytest.raises(TypeError):
        p.coordinates = "str"
    with pytest.raises(TypeError):
        p.x = "str"
    with pytest.raises(TypeError):
        p.y = "str"


def test_dla_empty_graph():
    g = gr.Graph()

    nodes = list(g.node_indices())
    assert len(nodes) == 0

    edges = list(g.edge_indices())
    assert len(edges) == 0


def test_dla_example_graph_nodes():
    g = gr.Graph.new_example_graph()

    nodes = list(g.node_indices())
    node_indices = [n.index() for n in nodes]
    particles = [g.node_weight(n) for n in nodes]

    assert len(nodes) == 3
    assert len(node_indices) == 3
    assert len(particles) == 3

    assert isinstance(g.node_indices(), gr.NodeIndices)
    assert isinstance(nodes[0], gr.NodeIndex)
    assert isinstance(particles[0], gr.Particle)

    assert node_indices == [0, 1, 2]

    assert particles[0].coordinates == [0, 0]
    assert particles[1].coordinates == [1, 0]
    assert particles[2].coordinates == [0, 1]


def test_dla_example_graph_edges():
    g = gr.Graph.new_example_graph()

    edges = list(g.edge_indices())
    nodes = list(g.node_indices())
    edge_indices = [e.index() for e in edges]
    edge_nodeindex_pairs = [g.edge_endpoints(e) for e in edges]
    edge_index_pairs = [(s.index(), t.index()) for s, t in edge_nodeindex_pairs]

    assert len(nodes) == 3
    assert len(edges) == 2

    assert isinstance(g.edge_indices(), gr.EdgeIndices)
    assert isinstance(edges[0], gr.EdgeIndex)
    assert isinstance(edge_nodeindex_pairs[0][0], gr.NodeIndex)

    assert len(edge_indices) == 2
    assert edge_indices == [0, 1]

    assert len(edge_nodeindex_pairs) == 2
    assert len(edge_index_pairs) == 2

    assert edge_index_pairs == [(0, 1), (0, 2)]
