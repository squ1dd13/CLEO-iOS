#include <memory>
#include <stdexcept>
#include <string>
#include <vector>
#include <os/log.h>

// #define SHOW_DEBUG_OVERLAY

#ifndef DEBUG_HEADER
#define DEBUG_HEADER

struct Debug {
    static std::vector<std::string> logStrings;

    template <typename... Args>
    static inline void logf(const std::string &format, Args... args) {
        // https://stackoverflow.com/a/26221725/8622854
        size_t size = snprintf(nullptr, 0, format.c_str(), args...) + 1;

        if(size <= 0) {
            throw std::runtime_error("Formatting error.");
        }

        std::unique_ptr<char[]> buf(new char[size]);
        snprintf(buf.get(), size, format.c_str(), args...);

#ifdef SHOW_DEBUG_OVERLAY
        logStrings.emplace_back(buf.get(), buf.get() + size - 1);
#endif
        os_log(OS_LOG_DEFAULT, "[CSiOS] %{public}s", std::string(buf.get(), buf.get() + size - 1).c_str());
    }

    template <typename... Args>
    static inline void assertf(bool condition, const std::string &format, Args... args) {
        if(!condition) {
            logf("err: " + format, args...);
        }
    }

    static inline bool needsUpdate() {
        return !logStrings.empty();
    }
};

#include <execinfo.h>  // for backtrace
#include <dlfcn.h>     // for dladdr
#include <cxxabi.h>    // for __cxa_demangle

#include <string>
#include <sstream>
#include <mach-o/dyld.h>
// This function produces a stack backtrace with demangled function & method names.
inline std::string Backtrace(int skip = 1)
{
	void *callstack[128];
	const int nMaxFrames = sizeof(callstack) / sizeof(callstack[0]);
	char buf[1024];
	int nFrames = backtrace(callstack, nMaxFrames);

    auto slide = _dyld_get_image_vmaddr_slide(0);
    for(int i = 0; i < nFrames; ++i) { 
        callstack[i] = (void *)(size_t(callstack[i]) - slide);
    }
	char **symbols = backtrace_symbols(callstack, nFrames);

	std::ostringstream trace_buf;
	for (int i = skip; i < nFrames; i++) {
		Dl_info info;
		if (dladdr(callstack[i], &info)) {
			char *demangled = NULL;
			int status;
			demangled = abi::__cxa_demangle(info.dli_sname, NULL, 0, &status);
			snprintf(buf, sizeof(buf), "%-3d %0*p %s + %zd\n",
					 i, 2 + sizeof(void*) * 2, callstack[i],
					 status == 0 ? demangled : info.dli_sname,
					 (char *)callstack[i] - (char *)info.dli_saddr);
			free(demangled);
		} else {
			snprintf(buf, sizeof(buf), "%-3d %0*p\n",
					 i, 2 + sizeof(void*) * 2, callstack[i]);
		}
		trace_buf << buf;

		snprintf(buf, sizeof(buf), "%s\n", symbols[i]);
		trace_buf << buf;
	}
	free(symbols);
	if (nFrames == nMaxFrames)
		trace_buf << "[truncated]\n";
	return trace_buf.str();
}

#endif