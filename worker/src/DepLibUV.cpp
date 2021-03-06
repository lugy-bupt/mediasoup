#define MS_CLASS "DepLibUV"
// #define MS_LOG_DEV_LEVEL 3

#include "DepLibUV.hpp"
#include "Logger.hpp"
#include <cstdlib> // std::abort()

DepLibUV::DepLibUV()
{
	// NOTE: Logger depends on this so we cannot log anything here.

	this->loop = new uv_loop_t;

	int err = uv_loop_init(this->loop);

	if (err != 0)
		MS_ABORT("libuv initialization failed");
}

uv_loop_t* DepLibUV::GetLoop()
{
	MS_TRACE();

	return this->loop;
}

DepLibUV::~DepLibUV()
{
	MS_TRACE();

	// This should never happen.
	if (this->loop != nullptr)
	{
		uv_loop_close(this->loop);
		delete this->loop;
	}
}

void DepLibUV::PrintVersion()
{
	MS_TRACE();

	MS_DEBUG_TAG(info, "libuv version: \"%s\"", uv_version_string());
}

void DepLibUV::RunLoop()
{
	MS_TRACE();

	// This should never happen.
	MS_ASSERT(this->loop != nullptr, "loop unset");

	uv_run(this->loop, UV_RUN_DEFAULT);
}
