#pragma once
#include "bem_rewrite.h"
namespace betterendfieldnext {
void CancelTextureCompression();
// True when the device can keep this texture as-is (format supported and inside
// the Android install size limits); the caller can then skip decoding it.
bool TextureCanBeKept(const BetterEndfieldNext::CustomModel::BemJson& texture,
    bool astc, unsigned supported_mask);
void ConvertInstalledTexture(const BetterEndfieldNext::CustomModel::BemJson& manifest,
    BetterEndfieldNext::CustomModel::BemJson& texture, std::vector<uint8_t>& bytes,
    const BetterEndfieldNext::CustomModel::BemJson& rules, bool astc,
    unsigned supported_mask,
    const std::function<void()>& checkpoint,
    const std::function<void(unsigned, unsigned, float)>& progress = {});
bool RepairNormalTexture(const BetterEndfieldNext::CustomModel::BemJson& manifest,
    BetterEndfieldNext::CustomModel::BemJson& texture, std::vector<uint8_t>& bytes, bool astc,
    const std::function<void()>& checkpoint,
    const std::function<void(unsigned, unsigned, float)>& progress = {});
}
