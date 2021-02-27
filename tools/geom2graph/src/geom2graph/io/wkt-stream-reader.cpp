#include "geom2graph/io/wkt-stream-reader.h"

#include <geos/geom/Geometry.h>
#include <geos/geom/GeometryFactory.h>
#include <geos/io/ParseException.h>
#include <log4cplus/logger.h>
#include <log4cplus/loggingmacros.h>

static log4cplus::Logger s_logger = log4cplus::Logger::getInstance("geom2graph.io.wkt");

namespace geom2graph::io {
WKTStreamReader::GeometryIterator::GeometryIterator(std::istream& input_stream,
                                                    const geos::geom::GeometryFactory& factory,
                                                    bool is_done) :
    m_is_at_end(is_done),
    m_is_past_end(is_done),
    m_input_stream(input_stream),
    m_wkt_reader(factory)
{
    // Need to prime the pump, so to speak.
    if (!m_is_past_end)
    {
        this->operator++();
    }
    // We consumed the entire istream, but failed to create a valid geometry.
    if (m_is_at_end && !m_current_value)
    {
        m_is_past_end = true;
    }
}

WKTStreamReader::GeometryIterator& WKTStreamReader::GeometryIterator::operator++()
{
    if (m_is_past_end)
    {
        // LOG4CPLUS_TRACE(s_logger, "Advancing GeometryIterator past the end.");
        return *this;
    }

    if (m_is_at_end && !m_is_past_end)
    {
        // LOG4CPLUS_TRACE(s_logger, "Advancing GeometryIterator to the end.");
        m_is_past_end = true;
        return *this;
    }

    std::string line;
    bool got_valid_geometry = false;
    do
    {
        std::getline(m_input_stream, line);
        // LOG4CPLUS_TRACE(s_logger, "Reading '" << line << "' from stream");
        m_is_at_end = m_input_stream.eof();
        try
        {
            m_current_value = m_wkt_reader.read(line);
            got_valid_geometry = true;
            LOG4CPLUS_DEBUG(s_logger, "Read geometry '" << m_current_value->toString() << "'");
        } catch (const geos::io::ParseException& e)
        {
            // Need to handle the case that this was the last line in the stream. >:(
            if (m_is_at_end)
            {
                m_is_past_end = true;
            } else
            {
                // Only print the warning when the line isn't the last line in the stream, because
                // if the stream ends in a newline (which it does under normal circumstances) we
                // "fail" to parse the last empty line.
                LOG4CPLUS_WARN(s_logger, "Failed to parse '" << line << "' as valid WKT geometry");
            }
        }
    } while (!got_valid_geometry && !m_is_at_end);

    return *this;
}

bool WKTStreamReader::GeometryIterator::operator==(const GeometryIterator& rhs) const
{
    // Because the wrapped data is ephemeral, iterators are equal unless we happen to know for sure
    // that they're not.
    return this->m_is_past_end == rhs.m_is_past_end && this->m_is_at_end == rhs.m_is_at_end;
}

bool WKTStreamReader::GeometryIterator::operator!=(const GeometryIterator& rhs) const
{
    return !(*this == rhs);
}
}  // namespace geom2graph::io
