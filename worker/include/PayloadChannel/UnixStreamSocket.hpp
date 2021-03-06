#ifndef MS_PAYLOAD_CHANNEL_UNIX_STREAM_SOCKET_HPP
#define MS_PAYLOAD_CHANNEL_UNIX_STREAM_SOCKET_HPP

#include "common.hpp"
#include "DepLibUV.hpp"
#include "PayloadChannel/Notification.hpp"
#include "PayloadChannel/Request.hpp"
#include "handles/UnixStreamSocket.hpp"
#include <json.hpp>

namespace PayloadChannel
{
	class ConsumerSocket : public ::UnixStreamSocket
	{
	public:
		class Listener
		{
		public:
			virtual DepLibUV* GetDepLibUV(ConsumerSocket* consumerSocket) = 0;
			virtual void OnConsumerSocketMessage(ConsumerSocket* consumerSocket, char* msg, size_t msgLen) = 0;
			virtual void OnConsumerSocketClosed(ConsumerSocket* consumerSocket) = 0;
		};

	public:
		ConsumerSocket(int fd, size_t bufferSize, Listener* listener);

		/* Pure virtual methods inherited from ::UnixStreamSocket. */
	public:
		void UserOnUnixStreamRead() override;
		void UserOnUnixStreamSocketClosed() override;

	private:
		// Passed by argument.
		Listener* listener{ nullptr };
		// Others.
		size_t msgStart{ 0u }; // Where the latest message starts.
	};

	class ProducerSocket : public ::UnixStreamSocket
	{
	public:
		class Listener
		{
		public:
			virtual DepLibUV* GetDepLibUV(ProducerSocket* producerSocket) = 0;
		};

	public:
		ProducerSocket(int fd, size_t bufferSize, Listener* listener);

		/* Pure virtual methods inherited from ::UnixStreamSocket. */
	public:
		void UserOnUnixStreamRead() override
		{
		}
		void UserOnUnixStreamSocketClosed() override
		{
		}
	};

	class UnixStreamSocket : public ConsumerSocket::Listener, public ProducerSocket::Listener
	{
	public:
		class Listener
		{
		public:
			virtual DepLibUV* GetDepLibUV(PayloadChannel::UnixStreamSocket* payloadChannel) = 0;
			virtual void OnPayloadChannelNotification(
			  PayloadChannel::UnixStreamSocket* payloadChannel,
			  PayloadChannel::Notification* notification) = 0;
			virtual void OnPayloadChannelRequest(
			  PayloadChannel::UnixStreamSocket* payloadChannel, PayloadChannel::Request* request) = 0;
			virtual void OnPayloadChannelClosed(PayloadChannel::UnixStreamSocket* payloadChannel) = 0;
		};

	public:
		explicit UnixStreamSocket(DepLibUV* depLibUV, int consumerFd, int producerFd);
		virtual ~UnixStreamSocket();

	public:
		void SetListener(Listener* listener);
		void Send(json& jsonMessage, const uint8_t* payload, size_t payloadLen);
		void Send(json& jsonMessage);

	private:
		void SendImpl(const void* nsPayload, size_t nsPayloadLen);

		/* Pure virtual methods inherited from ConsumerSocket::Listener. */
	public:
		DepLibUV* GetDepLibUV(ConsumerSocket* consumerSocket) override;
		void OnConsumerSocketMessage(ConsumerSocket* consumerSocket, char* msg, size_t msgLen) override;
		void OnConsumerSocketClosed(ConsumerSocket* consumerSocket) override;

		/* Pure virtual methods inherited from ProducerSocket::Listener. */
	public:
		DepLibUV* GetDepLibUV(ProducerSocket* producerSocket) override;

	private:
		// Passed by argument.
		DepLibUV* depLibUV{ nullptr };
		Listener* listener{ nullptr };
		// Others.
		ConsumerSocket consumerSocket;
		ProducerSocket producerSocket;
		PayloadChannel::Notification* ongoingNotification{ nullptr };
		PayloadChannel::Request* ongoingRequest{ nullptr };
	};
} // namespace PayloadChannel

#endif
