#include "geom2graph/io/tgf-graph-reader.h"
#include "geom2graph/io/wkt.h"

#include <geos/geom/GeometryFactory.h>

#include <sstream>

#include <gmock/gmock.h>
#include <gtest/gtest.h>

using namespace ::testing;
using geom2graph::io::from_wkt;

TEST(TGFGraphReaderTests, IstreamSanity)
{
    std::istringstream input{"asdf 3.14\n"};

    double pi = 0;
    input >> pi;

    EXPECT_TRUE(input.fail());
    EXPECT_EQ(pi, 0);

    // CLear the failure flags.
    input.clear();

    std::string misc;
    input >> misc;
    EXPECT_FALSE(input.fail());
    EXPECT_EQ(misc, "asdf");

    input >> pi;
    EXPECT_FALSE(input.fail());
    EXPECT_EQ(pi, 3.14);
}

TEST(TGFGraphReaderTests, IstreamReadRestofLineWithWhitespace)
{
    std::istringstream input{"0 POINT (0 0)\n"};
    std::size_t index = 0;
    input >> index;
    ASSERT_FALSE(input.fail());

    std::string label;
    std::getline(input, label);

    ASSERT_FALSE(input.fail());
    // Reads rest of line, including the space after the index.
    EXPECT_EQ(label, " POINT (0 0)");
}

TEST(TGFGraphReaderTests, NodesOnly)
{
    std::istringstream input{"0 POINT(0 0)\n1 POINT(1 1)\n2 POINT Z (2 2 2)\n#"};
    const auto factory = geos::geom::GeometryFactory::create();

    geom2graph::io::TGFGraphReader reader(input, *factory);
    auto graph = reader.read();

    const auto& nodes = graph.get_nodes();
    const auto& edges = graph.get_edge_pairs();

    ASSERT_THAT(nodes, SizeIs(3));
    ASSERT_THAT(edges, SizeIs(0));

    EXPECT_TRUE(nodes[0].point->equals(from_wkt("POINT(0 0)").get()));
    EXPECT_EQ(nodes[0].index, 0);
    EXPECT_THAT(nodes[0].adjacencies, SizeIs(0));
    EXPECT_TRUE(nodes[1].point->equals(from_wkt("POINT(1 1)").get()));
    EXPECT_EQ(nodes[1].index, 1);
    EXPECT_THAT(nodes[1].adjacencies, SizeIs(0));
    EXPECT_TRUE(nodes[2].point->equals(from_wkt("POINT Z(2 2 2)").get()));
    EXPECT_EQ(nodes[2].index, 2);
    EXPECT_THAT(nodes[2].adjacencies, SizeIs(0));
}

TEST(TGFGraphReaderTests, Simple2D)
{
    std::stringstream input{
        // clang-format off
        "0\tPOINT (0 0)\n"
        "1\tPOINT (1 1)\n"
        "#\n"
        "0\t1\n"
        // clang-format on
    };
    const auto factory = geos::geom::GeometryFactory::create();
    geom2graph::io::TGFGraphReader reader(input, *factory);
    auto graph = reader.read();

    const auto& nodes = graph.get_nodes();
    const auto& edges = graph.get_edge_pairs();

    ASSERT_THAT(nodes, SizeIs(2));
    ASSERT_THAT(edges, SizeIs(1));

    EXPECT_TRUE(nodes[0].point->equals(from_wkt("POINT(0 0)").get()));
    EXPECT_TRUE(nodes[1].point->equals(from_wkt("POINT(1 1)").get()));

    const auto& edge = edges[0];
    EXPECT_EQ(edge.first.index, 0);
    EXPECT_EQ(edge.second.index, 1);

    EXPECT_THAT(nodes[0].adjacencies, ElementsAre(1));
    EXPECT_THAT(nodes[1].adjacencies, ElementsAre(0));
}

TEST(TGFGraphReaderTests, DuplicateNode)
{
    std::stringstream input{
        // clang-format off
        "0\tPOINT (0 0)\n"
        "1\tPOINT (1 1)\n"
        "1\tPOINT (1 1)\n"
        "#\n"
        "0\t1\n"
        // clang-format on
    };
    const auto factory = geos::geom::GeometryFactory::create();
    geom2graph::io::TGFGraphReader reader(input, *factory);
    auto graph = reader.read();

    const auto& nodes = graph.get_nodes();
    const auto& edges = graph.get_edge_pairs();

    ASSERT_THAT(nodes, SizeIs(2));
    ASSERT_THAT(edges, SizeIs(1));

    EXPECT_TRUE(nodes[0].point->equals(from_wkt("POINT(0 0)").get()));
    EXPECT_TRUE(nodes[1].point->equals(from_wkt("POINT(1 1)").get()));

    const auto& edge = edges[0];
    EXPECT_EQ(edge.first.index, 0);
    EXPECT_EQ(edge.second.index, 1);

    EXPECT_THAT(nodes[0].adjacencies, ElementsAre(1));
    EXPECT_THAT(nodes[1].adjacencies, ElementsAre(0));
}

TEST(TGFGraphReaderTests, EdgesWithUnknownNodes)
{
    std::stringstream input{
        // clang-format off
        "0\tPOINT (0 0)\n"
        "1\tPOINT (1 1)\n"
        "#\n"
        "0\t1\n"
        "1\t9999\n"
        // clang-format on
    };

    const auto factory = geos::geom::GeometryFactory::create();
    geom2graph::io::TGFGraphReader reader(input, *factory);
    auto graph = reader.read();

    const auto& nodes = graph.get_nodes();
    const auto& edges = graph.get_edge_pairs();

    ASSERT_THAT(nodes, SizeIs(2));
    ASSERT_THAT(edges, SizeIs(1));

    const auto& edge = edges[0];
    EXPECT_EQ(edge.first.index, 0);
    EXPECT_EQ(edge.second.index, 1);

    EXPECT_THAT(nodes[0].adjacencies, ElementsAre(1));
    EXPECT_THAT(nodes[1].adjacencies, ElementsAre(0));
}

TEST(TGFGraphReaderTests, DuplicateEdge)
{
    std::stringstream input{
        // clang-format off
        "0\tPOINT (0 0)\n"
        "1\tPOINT (1 1)\n"
        "#\n"
        "0\t1\n"
        "0\t1\n"
        "1\t0\n"
        "1\t0\n"
        // clang-format on
    };

    const auto factory = geos::geom::GeometryFactory::create();
    geom2graph::io::TGFGraphReader reader(input, *factory);
    auto graph = reader.read();

    const auto& nodes = graph.get_nodes();
    const auto& edges = graph.get_edge_pairs();

    ASSERT_THAT(nodes, SizeIs(2));
    ASSERT_THAT(edges, SizeIs(1));

    const auto& edge = edges[0];
    EXPECT_EQ(edge.first.index, 0);
    EXPECT_EQ(edge.second.index, 1);

    EXPECT_THAT(nodes[0].adjacencies, ElementsAre(1));
    EXPECT_THAT(nodes[1].adjacencies, ElementsAre(0));
}

TEST(TGFGraphReaderTests, MissingNodes)
{
    std::stringstream input{
        // clang-format off
        "0\tPOINT (0 0)\n"
            "2\tPOINT (2 2)\n"
            "#\n"
            "0\t1\n"
        // clang-format on
    };

    const auto factory = geos::geom::GeometryFactory::create();
    geom2graph::io::TGFGraphReader reader(input, *factory);
    auto graph = reader.read();

    const auto& nodes = graph.get_nodes();
    const auto& edges = graph.get_edge_pairs();

    ASSERT_THAT(nodes, SizeIs(1));
    ASSERT_THAT(edges, SizeIs(0));

    EXPECT_TRUE(nodes[0].point->equals(from_wkt("POINT(0 0)").get()));
}

TEST(TGFGraphReaderTests, OutOfOrderNodes)
{
    std::stringstream input{
        // clang-format off
        "0\tPOINT (0 0)\n"
        "2\tPOINT (2 2)\n" // This node should get skipped.
        "1\tPOINT (1 1)\n"
        "#\n"
        "0\t1\n"
        // clang-format on
    };

    const auto factory = geos::geom::GeometryFactory::create();
    geom2graph::io::TGFGraphReader reader(input, *factory);
    auto graph = reader.read();

    const auto& nodes = graph.get_nodes();
    const auto& edges = graph.get_edge_pairs();

    ASSERT_THAT(nodes, SizeIs(2));
    ASSERT_THAT(edges, SizeIs(1));

    EXPECT_TRUE(nodes[0].point->equals(from_wkt("POINT(0 0)").get()));
    EXPECT_TRUE(nodes[1].point->equals(from_wkt("POINT(1 1)").get()));

    const auto& edge = edges[0];
    EXPECT_EQ(edge.first.index, 0);
    EXPECT_EQ(edge.second.index, 1);

    EXPECT_THAT(nodes[0].adjacencies, ElementsAre(1));
    EXPECT_THAT(nodes[1].adjacencies, ElementsAre(0));
}

TEST(TGFGraphReaderTests, IgnoresBlankLines)
{
    std::stringstream input{
        // clang-format off
        "\n"
        "0\tPOINT (0 0)\r\n"
        "\r\n"
        "  \t \n"
        "\n\r"
        "1\tPOINT (1 1)\n"
        "\n"
        "\n#\n\n"
        "\n"
        "0\t1\n"
        "\n"
        // clang-format on
    };

    const auto factory = geos::geom::GeometryFactory::create();
    geom2graph::io::TGFGraphReader reader(input, *factory);
    auto graph = reader.read();

    const auto& nodes = graph.get_nodes();
    const auto& edges = graph.get_edge_pairs();

    ASSERT_THAT(nodes, SizeIs(2));
    ASSERT_THAT(edges, SizeIs(1));

    EXPECT_TRUE(nodes[0].point->equals(from_wkt("POINT(0 0)").get()));
    EXPECT_TRUE(nodes[1].point->equals(from_wkt("POINT(1 1)").get()));

    const auto& edge = edges[0];
    EXPECT_EQ(edge.first.index, 0);
    EXPECT_EQ(edge.second.index, 1);

    EXPECT_THAT(nodes[0].adjacencies, ElementsAre(1));
    EXPECT_THAT(nodes[1].adjacencies, ElementsAre(0));
}

TEST(TGFGraphReaderTests, IgnoresGarbageLines)
{
    std::stringstream input{
        // clang-format off
        "0\tPOINT (0 0)\r\n"
        "1\tPOINT (1 1)\n"
        "THIS IS GARBAGE \t\r \n"
        "#\n"
        "0\t1\n"
        // clang-format on
    };

    const auto factory = geos::geom::GeometryFactory::create();
    geom2graph::io::TGFGraphReader reader(input, *factory);
    auto graph = reader.read();

    const auto& nodes = graph.get_nodes();
    const auto& edges = graph.get_edge_pairs();

    ASSERT_THAT(nodes, SizeIs(2));
    ASSERT_THAT(edges, SizeIs(1));

    EXPECT_TRUE(nodes[0].point->equals(from_wkt("POINT(0 0)").get()));
    EXPECT_TRUE(nodes[1].point->equals(from_wkt("POINT(1 1)").get()));

    const auto& edge = edges[0];
    EXPECT_EQ(edge.first.index, 0);
    EXPECT_EQ(edge.second.index, 1);

    EXPECT_THAT(nodes[0].adjacencies, ElementsAre(1));
    EXPECT_THAT(nodes[1].adjacencies, ElementsAre(0));
}

TEST(TGFGraphReaderTests, InvalidWKTLabels)
{
    std::stringstream input{
        // clang-format off
        "0\tPOINT (0 0)\r\n"
        "1\tPOINT (1 1)\n"
        "2\tPOINT (1,1)\n" // invalid WKT
        "#\n"
        "0\t1\n"
        // clang-format on
    };

    const auto factory = geos::geom::GeometryFactory::create();
    geom2graph::io::TGFGraphReader reader(input, *factory);
    auto graph = reader.read();

    const auto& nodes = graph.get_nodes();
    const auto& edges = graph.get_edge_pairs();

    ASSERT_THAT(nodes, SizeIs(2));
    ASSERT_THAT(edges, SizeIs(1));

    EXPECT_TRUE(nodes[0].point->equals(from_wkt("POINT(0 0)").get()));
    EXPECT_TRUE(nodes[1].point->equals(from_wkt("POINT(1 1)").get()));

    const auto& edge = edges[0];
    EXPECT_EQ(edge.first.index, 0);
    EXPECT_EQ(edge.second.index, 1);

    EXPECT_THAT(nodes[0].adjacencies, ElementsAre(1));
    EXPECT_THAT(nodes[1].adjacencies, ElementsAre(0));
}

TEST(TGFGraphReaderTests, ValidNonPointWKTLabels)
{
    std::stringstream input{
        // clang-format off
        "0\tPOINT (0 0)\n"
        "1\tLINESTRING (0 0, 1 1, 2 2)\n"
        "1\tPOINT (1 1)\n"
        "#\n"
        "0\t1\n"
        // clang-format on
    };
    const auto factory = geos::geom::GeometryFactory::create();
    geom2graph::io::TGFGraphReader reader(input, *factory);
    auto graph = reader.read();

    const auto& nodes = graph.get_nodes();
    const auto& edges = graph.get_edge_pairs();

    ASSERT_THAT(nodes, SizeIs(2));
    ASSERT_THAT(edges, SizeIs(1));

    EXPECT_TRUE(nodes[0].point->equals(from_wkt("POINT(0 0)").get()));
    EXPECT_TRUE(nodes[1].point->equals(from_wkt("POINT(1 1)").get()));

    const auto& edge = edges[0];
    EXPECT_EQ(edge.first.index, 0);
    EXPECT_EQ(edge.second.index, 1);

    EXPECT_THAT(nodes[0].adjacencies, ElementsAre(1));
    EXPECT_THAT(nodes[1].adjacencies, ElementsAre(0));
}
