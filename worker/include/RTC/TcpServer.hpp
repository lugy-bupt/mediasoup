#ifndef MS_RTC_TCP_SERVER_HPP
#define MS_RTC_TCP_SERVER_HPP

#include "common.hpp"
#include "DepLibUV.hpp"
#include "RTC/TcpConnection.hpp"
#include "handles/TcpConnection.hpp"
#include "handles/TcpServer.hpp"
#include <string>

namespace RTC
{
	class TcpServer : public ::TcpServer
	{
	public:
		class Listener
		{
		public:
			virtual DepLibUV* GetDepLibUV(RTC::TcpServer* tcpServer) = 0;
			virtual void OnRtcTcpConnectionClosed(
			  RTC::TcpServer* tcpServer, RTC::TcpConnection* connection) = 0;
		};

	public:
		TcpServer(Listener* listener, RTC::TcpConnection::Listener* connListener, std::string& ip);
		~TcpServer() override;

		/* Pure virtual methods inherited from ::TcpServer. */
	public:
		DepLibUV* GetDepLibUV(::TcpConnection* connection) override;
		void UserOnTcpConnectionAlloc() override;
		void UserOnTcpConnectionClosed(::TcpConnection* connection) override;

	private:
		// Passed by argument.
		Listener* listener{ nullptr };
		RTC::TcpConnection::Listener* connListener{ nullptr };
	};
} // namespace RTC

#endif
