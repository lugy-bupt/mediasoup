#define CATCH_CONFIG_RUNNER

#include "helpers.hpp"
#include "DepLibSRTP.hpp"
#include "DepLibUV.hpp"
#include "DepLibWebRTC.hpp"
#include "DepOpenSSL.hpp"
#include "DepUsrSCTP.hpp"
#include "LogLevel.hpp"
#include "Settings.hpp"
#include "Utils.hpp"
#include <catch.hpp>
#include <cstdlib> // std::getenv()

int main(int argc, char* argv[])
{
	LogLevel logLevel{ LogLevel::LOG_NONE };

	// Get logLevel from ENV variable.
	if (std::getenv("MS_TEST_LOG_LEVEL"))
	{
		if (std::string(std::getenv("MS_TEST_LOG_LEVEL")) == "debug")
			logLevel = LogLevel::LOG_DEBUG;
		else if (std::string(std::getenv("MS_TEST_LOG_LEVEL")) == "warn")
			logLevel = LogLevel::LOG_WARN;
		else if (std::string(std::getenv("MS_TEST_LOG_LEVEL")) == "error")
			logLevel = LogLevel::LOG_ERROR;
	}

	Settings::configuration.logLevel = logLevel;

	auto* depLibUV = helpers::getDepLibUV();

	// Initialize static stuff.
	DepOpenSSL::ClassInit();
	DepLibSRTP::ClassInit();
	DepUsrSCTP::ClassInit(depLibUV);
	DepLibWebRTC::ClassInit();
	Utils::Crypto::ClassInit();

	int status = Catch::Session().run(argc, argv);

	delete depLibUV;

	// Free static stuff.
	DepLibSRTP::ClassDestroy();
	Utils::Crypto::ClassDestroy();
	DepLibWebRTC::ClassDestroy();
	DepUsrSCTP::ClassDestroy();

	return status;
}
