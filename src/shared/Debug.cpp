//
// Created by squ1dd13 on 09/11/2020.
//

#include "Debug.h"

void Log::Commit(const std::string &s) {
    // Open the log file, clearing it if it isn't empty.
    static std::ofstream stream = std::ofstream(
        "/var/mobile/Documents/CSiOS.log",
        std::ofstream::out | std::ofstream::trunc
    );

    if (log.size() >= messageLimit) {
        // Remove the oldest message to make room for the newest.
        log.pop_front();
    }

    stream << s << '\n';
    stream.flush();
    log.push_back(s);
    updated = true;
}

std::deque<std::string> Log::log;
bool Log::updated = false;