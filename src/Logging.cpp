//
// Created by squ1dd13 on 30/12/2020.
//

#include "Logging.h"

#include <netinet/in.h>
#include <sys/socket.h>
#include <unistd.h>
#include <arpa/inet.h>
#include <iostream>

#define LOG_SERVER_IP "192.168.1.183"
#define LOG_SERVER_PORT 11909

void LogErr(const std::string &&s) {
    std::cerr << "SendBuf(): " << s << "! (errno = " << errno << ": '" << strerror(errno) << "')\n";
}

int sockfd = -1;
sockaddr_in addr {};

bool EnsureSocketOpen() {
    if (sockfd != -1) {
        return true;
    }

    sockfd = socket(PF_INET, SOCK_DGRAM, IPPROTO_UDP);

    if (sockfd <= 0) {
        LogErr("Failed to open socket");
        return false;
    }

    int broadcastEnable = 1;
    int r = setsockopt(sockfd, SOL_SOCKET, SO_BROADCAST, &broadcastEnable, sizeof(broadcastEnable));

    if (r) {
        LogErr("Failed to put socket in broadcast mode");
        close(sockfd);
        return false;
    }

    addr = sockaddr_in {
        0,
        AF_INET,
        htons(LOG_SERVER_PORT),
        0
    };

    inet_pton(AF_INET, LOG_SERVER_IP, &addr.sin_addr);

    return true;
}

void CloseSocket() {
    close(sockfd);
    sockfd = -1;
}

void SendBuf(void *data, size_t length) {
    EnsureSocketOpen();

    if (sendto(sockfd, data, length, 0, (sockaddr *)&addr, sizeof(addr)) < 0) {
        LogErr("Failed to send log message over socket");
        CloseSocket();

        return;
    }
}

[[maybe_unused]]
__attribute__((destructor))
void LogDestruct() {
    LogInfo("Closing socket. Bye!");

    if (sockfd != -1) {
        close(sockfd);
    }
}