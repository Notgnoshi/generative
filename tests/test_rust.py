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
