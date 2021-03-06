#ifndef MS_DEP_LIBUV_HPP
#define MS_DEP_LIBUV_HPP

#include "common.hpp"
#include "DepLibUV.hpp"
#include <uv.h>

class DepLibUV
{
public:
	DepLibUV();
	~DepLibUV();
	static void PrintVersion();
	void RunLoop();
	uv_loop_t* GetLoop();
	static uint64_t GetTimeMs()
	{
		return static_cast<uint64_t>(uv_hrtime() / 1000000u);
	}
	static uint64_t GetTimeUs()
	{
		return static_cast<uint64_t>(uv_hrtime() / 1000u);
	}
	static uint64_t GetTimeNs()
	{
		return uv_hrtime();
	}
	// Used within libwebrtc dependency which uses int64_t values for time
	// representation.
	static int64_t GetTimeMsInt64()
	{
		return static_cast<int64_t>(DepLibUV::GetTimeMs());
	}
	// Used within libwebrtc dependency which uses int64_t values for time
	// representation.
	static int64_t GetTimeUsInt64()
	{
		return static_cast<int64_t>(DepLibUV::GetTimeUs());
	}

private:
	uv_loop_t* loop;
};

#endif
