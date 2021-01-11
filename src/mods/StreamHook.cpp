//
// Created by squ1dd13 on 08/11/2020.
//

#include "StreamHook.h"

#include <pthread.h>
#include <dispatch/dispatch.h>
#include <sys/stat.h>
#include "Core.h"
#include "../shared/Memory.h"

struct Segments {
private:
    uint32 value;

public:
    inline uint32 segments() {
        return value;
    }

    inline uint32 bytes() {
        return value * 2048;
    }

    void setSegments(uint32 segments) {
        value = segments;
    }

    void setBytes(uint32 bytes) {
        // Round up.
        value = (bytes + 2047) / 2048 * 2048;
    }
} squished;

static_assert(sizeof(Segments) == 4, "bad");

struct CdStream {
    Segments offset;
    Segments size;
    void *buffer;

    uint8 _pad1;

    bool accessSemaphore;
    bool busy;

    uint8 _pad2;

    uint32 errorCode;
    dispatch_semaphore_t semaphore;
    pthread_mutex_t *mutex;
    FILE *stream;
} squished;

struct IndexQueue {
    uint32 *data;
    uint32 front;
    uint32 back;
    uint32 length;
} squished;

static_assert(sizeof(CdStream) == 48, "bad");

void SetFilePos(FILE *file, uint64 pos) {
    Memory::call(0x1004e51dc, file, pos);
}

uint64 ReadBytes(FILE *file, void *buf, uint32 count) {
    auto func = Memory::slid<uint64 (*)(FILE *, void *, uint32)>(0x1004e5300);
    return func(file, buf, count);
}

void SignalSemaphore(void *semaphore) {
    Memory::call(0x1004e8b5c, semaphore);
}

void *AllocateAligned(uint32 size, uint32 alignValue) {
//    void *result = 1003a13f8
    return Memory::call<void *>(0x1003a13f8, size, alignValue);
}

/**
 * Reimplementation of game code, but with some of our own modifications.
 */
void CdStreamThread(void *) {
    auto *semaphore = Memory::fetch<dispatch_semaphore_t *>(0x1006ac8e0);
    auto *streams = Memory::fetch<CdStream *>(0x100939118);
    auto *queue = Memory::slid<IndexQueue *>(0x100939120);

    auto streamingBufferSize = Memory::fetch<uint32>(0x10072d320);
    void *streamingBuffer = Memory::fetch<void *>(0x10072d328);

    Log::Print("Stream thread running");

    while (true) {
        dispatch_semaphore_wait(*semaphore, DISPATCH_TIME_FOREVER);

        int streamIndex = queue->front == queue->back ? -1 : int(queue->data[queue->front]);

        CdStream *stream = &streams[streamIndex];
        stream->busy = true;

        if (!stream->errorCode) {
            FILE *file = nullptr;
            uint32 len = stream->size.bytes();
//            1001323c8
            if (false && stream->offset.segments() == 88827) {
                // clover.dff
                struct stat st {};
                stat("/var/mobile/Documents/clover.dff", &st);
                len = st.st_size;

                file = fopen("/var/mobile/Documents/clover.dff", "rb");
                if (!file) {
                    Log::Print("file did not open");
                } else {
                    Log::Print("file opened, len = %d", len);
                }

                int err = ReadBytes((FILE *)&file, stream->buffer, len);
                stream->errorCode = err ? 0xfe : 0;

                fclose(file);
            } else {
                SetFilePos(stream->stream, stream->offset.bytes());

                int err = ReadBytes(stream->stream, stream->buffer, len);
                stream->errorCode = err ? 0xfe : 0;
            }

            if (stream->errorCode) {
                Log::Print("stream read error!");
            }
        }

        if (queue->front != queue->back) {
            uint32 iv2 = 0;
            if (queue->length) {
                iv2 = (queue->front + 1) / queue->length;
            }

            queue->front = (queue->front + 1) - iv2 * queue->length;
        }

        pthread_mutex_lock(stream->mutex);
        stream->size.setBytes(0);

        if (stream->accessSemaphore) {
            SignalSemaphore(stream->semaphore);
        }

        stream->busy = false;
        pthread_mutex_unlock(stream->mutex);
    }
}

HookFunction(StreamingThread, 0x100177dac, {
    CdStreamThread(x);
}, void, void *x);