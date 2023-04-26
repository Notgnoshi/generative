#include "generative/io/tgf-graph-reader.h"

#include "generative/io/wkt.h"

#include <geos/geom/Point.h>
#include <geos/io/ParseException.h>
#include <log4cplus/logger.h>
#include <log4cplus/loggingmacros.h>

#include <sstream>

static auto s_logger = log4cplus::Logger::getInstance("generative.io.tgfreader");

namespace generative::io {

generative::noding::GeometryGraph TGFGraphReader::read() noexcept
{
    for (std::string line; std::getline(m_input, line);)
    {
        this->read(line);
    }

    return {std::move(m_nodes_vec), m_factory};
}

void TGFGraphReader::read(const std::string& line) noexcept
{
    if (m_reading_nodes)
    {
        read_node(line);
    } else
    {
        read_edge(line);
    }
}

// Taken from https://stackoverflow.com/a/21174979/3704977 and modified to _not_ preserve the
// deleter.
template<typename Derived_t, typename Base_t>
static std::unique_ptr<Derived_t> dynamic_unique_ptr_cast(std::unique_ptr<Base_t>&& p)
{
    if (auto* result = dynamic_cast<Derived_t*>(p.get()))
    {
        (void)p.release();
        return std::unique_ptr<Derived_t>(result);
    }
    return std::unique_ptr<Derived_t>(nullptr);
}

static std::string trim(const std::string& str, const std::string& whitespace = " \t\n")
{
    const auto str_begin = str.find_first_not_of(whitespace);
    if (str_begin == std::string::npos)
    {
        return "";  // no content
    }

    const auto str_end = str.find_last_not_of(whitespace);
    const auto str_range = str_end - str_begin + 1;

    return str.substr(str_begin, str_range);
}

void TGFGraphReader::read_node(const std::string& _line) noexcept
{
    const auto line = trim(_line);
    std::istringstream input(line);

    if (line == "#")
    {
        LOG4CPLUS_INFO(s_logger, "Finished reading " << m_nodes_list.size() << " nodes");
        m_reading_nodes = false;
        m_nodes_vec.reserve(m_nodes_list.size());
        std::move(m_nodes_list.begin(), m_nodes_list.end(), std::back_inserter(m_nodes_vec));
        return;
    }

    std::size_t index = 0;
    std::string label;

    input >> index;
    std::getline(input, label);

    if (input.fail())
    {
        LOG4CPLUS_WARN(s_logger, "Failed to read node from line '" << line << "'");
        return;
    }

    //! @todo If we fail to parse node N, we will fail to parse node N+1, even if it is well-formed
    //! until we parse node N. I think to fix this, we need to use a std::unordered_map<size_t,
    //! Node>, and collapse it into a vector once both the nodes and the edges are known. This
    //! removes the requirement of node labels 1...N without gaps. Doing so will also require
    //! changing the index to be non-const in the Node definition.
    if (index > m_nodes_list.size())
    {
        LOG4CPLUS_WARN(s_logger,
                       "Attempted to read node index " << index << " before expected index "
                                                       << m_nodes_list.size());
        return;
    }
    if (index < m_nodes_list.size())
    {
        LOG4CPLUS_WARN(s_logger,
                       "Attempted to read duplicate index " << index << ". Expected index "
                                                            << m_nodes_list.size());
        return;
    }

    std::unique_ptr<geos::geom::Geometry> geometry = nullptr;
    try
    {
        geometry = m_reader.read(label);
    } catch (const geos::io::ParseException& e)
    {
        LOG4CPLUS_WARN(s_logger, "Failed to parse node label '" << label << "' as WKT");
        return;
    }

    if (geometry->getGeometryTypeId() != geos::geom::GeometryTypeId::GEOS_POINT)
    {
        LOG4CPLUS_WARN(s_logger, "Only POINT node labels are supported. Got '" << geometry << "'");
        return;
    }

    auto point = dynamic_unique_ptr_cast<geos::geom::Point>(std::move(geometry));
    if (!point)
    {
        LOG4CPLUS_ERROR(s_logger, "Failed to dynamic cast geometry '" << label << "' to point");
        return;
    }

    LOG4CPLUS_DEBUG(s_logger, "Adding node " << point << " at index " << index);
    m_nodes_list.emplace_back(index, std::move(point));
}

void TGFGraphReader::read_edge(const std::string& _line) noexcept
{
    const auto line = trim(_line);
    std::istringstream input(line);

    std::size_t src = 0;
    std::size_t dst = 0;

    input >> src >> dst;

    if (input.fail())
    {
        LOG4CPLUS_WARN(s_logger, "Failed to parse edge '" << line << "'");
        return;
    }

    if (src >= m_nodes_vec.size() || dst >= m_nodes_vec.size())
    {
        LOG4CPLUS_WARN(s_logger,
                       "Failed to add edge '" << src << " -> " << dst << "' because there are only "
                                              << m_nodes_vec.size() << " nodes");
        return;
    }

    //! @todo We need a way of guaranteeing that m_nodes_vec[src].index == src.
    m_nodes_vec[src].adjacencies.emplace(dst);
    m_nodes_vec[dst].adjacencies.emplace(src);
}
}  // namespace generative::io
