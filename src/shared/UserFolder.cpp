//
// Created by squ1dd13 on 08/11/2020.
//

#include "UserFolder.h"
#include <vector>
#include <string>
#include <dirent.h>
#include <map>

// TODO: Leave uninitialised until game load sequence.
Directory userFolder("/var/mobile/Documents/CS");

Directory::Directory(const std::string &path) {
    fullPath = path;
    DIR *directory = opendir(path.c_str());

    if (!directory) {
        return;
    }

    dirent *entry;
    while ((entry = readdir(directory))) {
        std::string entryPath = path + '/' + entry->d_name;

        if (entry->d_type == DT_LNK)
            continue;
        if (entry->d_type == DT_DIR) {
            std::string name(entry->d_name);
            if (name == ".." || name == ".")
                continue;

            directories[name] = Directory(entryPath);
            continue;
        }

        files.push_back(entryPath);
    }

    closedir(directory);
}

void Directory::FindAllOfType(Directory::FileType type, std::vector<File> &found) {
    std::string ext = GetFileTypeExtension(type);
    for (auto &path : files) {
        if (path.ends_with(ext)) {
            found.emplace_back(path, type);
        }
    }

    for (auto &pair : directories) {
        pair.second.FindAllOfType(type, found);
    }
}

std::string Directory::GetFileTypeExtension(Directory::FileType fileType) {
    static std::map<Directory::FileType, std::string> extensions = {
        { FileType::AndroidRunningScript, ".csa" },
        { FileType::AndroidInvokedScript, ".csi" },
        { FileType::WindowsScript,        ".cs" },
        { FileType::TextExtension,        ".fxt" },
    };

    return extensions[fileType];
}

Directory &Directory::operator[](const std::string &s) {
    return directories[s];
}

Directory Directory::operator[](const std::string &s) const {
    return directories.at(s);
}

bool Directory::HasChild(const std::string &sub) const {
    return directories.contains(fullPath + '/' + sub);
}

std::FILE *Directory::File::Open(const char *mode) const {
    return std::fopen(fullPath.c_str(), mode);
}

Directory::File::File(const std::string &path, Directory::FileType typ) {
    fullPath = path;
    type = typ;
}
