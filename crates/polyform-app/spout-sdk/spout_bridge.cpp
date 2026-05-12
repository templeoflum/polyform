// spout_bridge.cpp — Thin extern "C" wrapper around SpoutDX for Rust FFI.
// Part of the Polyform project. This file is original work, not part of the Spout2 SDK.

#include "SpoutDX.h"
#include <cstring>

extern "C" {

void* spout_sender_create() {
    return static_cast<void*>(new spoutDX());
}

bool spout_sender_open(void* ptr, const char* name, unsigned int width, unsigned int height) {
    auto* dx = static_cast<spoutDX*>(ptr);

    // On multi-GPU systems (e.g. Intel iGPU + NVIDIA dGPU), select the
    // discrete GPU so the shared texture handle is valid for receivers
    // (Resolume, OBS, etc.) running on the same adapter.
    int numAdapters = dx->GetNumAdapters();
    for (int i = 0; i < numAdapters; i++) {
        char adapterName[256]{};
        dx->GetAdapterName(i, adapterName, 256);
        if (strstr(adapterName, "NVIDIA") || strstr(adapterName, "AMD") ||
            strstr(adapterName, "Radeon")) {
            dx->SetAdapter(i);
            break;
        }
    }

    if (!dx->OpenDirectX11()) {
        return false;
    }
    dx->SetSenderName(name);
    return true;
}

bool spout_sender_send_image(void* ptr, const unsigned char* data,
                              unsigned int width, unsigned int height) {
    auto* dx = static_cast<spoutDX*>(ptr);
    return dx->SendImage(data, width, height);
}

void spout_sender_close(void* ptr) {
    auto* dx = static_cast<spoutDX*>(ptr);
    dx->ReleaseSender();
    dx->CloseDirectX11();
}

void spout_sender_destroy(void* ptr) {
    auto* dx = static_cast<spoutDX*>(ptr);
    delete dx;
}

} // extern "C"
