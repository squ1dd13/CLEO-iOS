//
// Created by squ1dd13 on 08/11/2020.
//

#pragma once

//#include <set>
#include <map>
#include <string>
#include <vector>

class Directory {
public:
    enum class FileType {
        AndroidRunningScript, // .csa
        AndroidInvokedScript, // .csi
        WindowsScript,        // .cs
        TextExtension,        // .fxt
    };

    std::vector<std::string> files;
    std::map<std::string, Directory> directories;
    std::string fullPath;

    struct File {
        FileType type;
        std::string fullPath;

        explicit File(const std::string &path, FileType typ);

        std::FILE *Open(const char *mode = "rb") const;
    };

    Directory() = default;
    explicit Directory(const std::string &path);

    bool HasChild(const std::string &sub) const;
    void FindAllOfType(FileType type, std::vector<File> &found);

    Directory &operator[](const std::string &s);
    Directory operator[](const std::string &s) const;

    static std::string GetFileTypeExtension(FileType fileType);
};

extern Directory userFolder;