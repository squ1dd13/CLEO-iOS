//
// Created by squ1dd13 on 08/11/2020.
//
// RW function implementations.

#include "shared/Memory.h"
#include "other/RenderWare.h"

RwTexture *RwTextureCreate(RwRaster *raster) {
    return Memory::call<RwTexture *>(0x1000fce78, raster);
}

bool RwTextureDestroy(RwTexture *texture) {
    return Memory::call<bool>(0x1000fcd98, texture);
}

RwRaster *RwRasterCreate(int32 width, int32 height, int32 depth, uint32 flags) {
    return Memory::call<RwRaster *>(0x1000fbe08, width, height, depth, flags);
}

bool RwRasterDestroy(RwRaster *raster) {
    return Memory::call<bool>(0x1000fbb90, raster);
}