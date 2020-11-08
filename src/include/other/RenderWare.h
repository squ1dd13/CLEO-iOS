//
// Created by squ1dd13 on 08/11/2020.
//

#pragma once

#include "other/Types.h"

// Structures taken from https://github.com/petrgeorgievsky/sa-render/blob/master/Render/RenderWare.h
//  created by DK22Pac.
// Some have been modified for the iOS version of the game, or to stop the need for defining more structures.
// A few offsets may be incorrect due to padding, but I can't manually pad until I know where all the fields are.

struct RwLLLink {
    RwLLLink *next;
    RwLLLink *prev;
};

struct RwLinkList {
    RwLLLink link;
};

struct RwRaster {
    RwRaster *parent;
    uint8 *cpPixels;
    uint8 *palette;
    int32 width, height, depth;
    int32 stride;
    int16 nOffsetX, nOffsetY;
    uint8 cType;
    uint8 cFlags;
    uint8 privateFlags;
    uint8 cFormat;
    uint8 *originalPixels;
    int32 originalWidth;
    int32 originalHeight;
    int32 originalStride;

    /* Native struct goes here */
};

struct RwObject {
    uint8 type;
    uint8 subType;
    uint8 flags;
    uint8 privateFlags;
    void *parent_RwFrame;
};

struct RwTexDictionary {
    RwObject object;
    RwLinkList texturesInDict;
    RwLLLink lInInstance;
    RwTexDictionary *parent;
};

struct RwTexture {
    RwRaster *raster;
    RwTexDictionary *dict;
    RwLLLink lInDictionary;
    char name[32];
    char mask[32];
    uint32 filterAddressing;
    int32 refCount;
    uint8 maxAnisotropy;

    uint8 pad[3];
};

// https://github.com/DK22Pac/plugin-sdk/blob/master/plugin_III/game_III/RenderWare.h
RwTexture *RwTextureCreate(RwRaster *raster);
bool RwTextureDestroy(RwTexture *texture);

RwRaster *RwRasterCreate(int32 width, int32 height, int32 depth, uint32 flags);
bool RwRasterDestroy(RwRaster *raster);

