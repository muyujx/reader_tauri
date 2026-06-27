/**
 * 构建 Android 资源脚本
 * 将前端构建产物和图标复制到 Android 项目目录
 * 图标使用 sharp 直接生成，确保 android_fg_scale 生效
 */
const fs = require('fs');
const path = require('path');
const sharp = require('sharp');

const root = 'src-tauri';
const srcDist = 'dist';
const destAssets = `${root}/gen/android/app/src/main/assets`;

// 复制目录
function copyDir(src, dest) {
    if (!fs.existsSync(dest)) {
        fs.mkdirSync(dest, { recursive: true });
    }
    fs.readdirSync(src).forEach(file => {
        const srcPath = path.join(src, file);
        const destPath = path.join(dest, file);
        if (fs.statSync(srcPath).isDirectory()) {
            copyDir(srcPath, destPath);
        } else {
            fs.copyFileSync(srcPath, destPath);
        }
    });
}

// 写入文件（自动创建目录）
function writeFile(filePath, content) {
    const dir = path.dirname(filePath);
    if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
    }
    fs.writeFileSync(filePath, content);
}

// Android mipmap 尺寸映射
const MIPMAP_SIZES = { mdpi: 48, hdpi: 72, xhdpi: 96, xxhdpi: 144, xxxhdpi: 192 };

// 读取图标配置清单
function readManifest() {
    const manifestPath = path.resolve(`${root}/icons/app-icon-manifest.json`);
    if (fs.existsSync(manifestPath)) {
        return JSON.parse(fs.readFileSync(manifestPath, 'utf-8'));
    }
    return null;
}

// 裁剪前景图的透明边距，返回 { buffer, width, height }
async function trimTransparent(fgPath) {
    const { data, info } = await sharp(fgPath).raw().ensureAlpha().toBuffer({ resolveWithObject: true });
    let minX = info.width, minY = info.height, maxX = 0, maxY = 0;
    let hasContent = false;
    for (let y = 0; y < info.height; y++) {
        for (let x = 0; x < info.width; x++) {
            if (data[(y * info.width + x) * 4 + 3] > 10) {
                if (x < minX) minX = x;
                if (y < minY) minY = y;
                if (x > maxX) maxX = x;
                if (y > maxY) maxY = y;
                hasContent = true;
            }
        }
    }
    if (!hasContent) {
        return sharp(fgPath).toBuffer().then(b => ({ buffer: b, width: info.width, height: info.height }));
    }
    const cropW = maxX - minX + 1, cropH = maxY - minY + 1;
    const buffer = await sharp(fgPath)
        .extract({ left: minX, top: minY, width: cropW, height: cropH })
        .toBuffer();
    return { buffer, width: cropW, height: cropH };
}

// 生成自适应图标前景图：先裁剪透明边距，再按 fgScale 缩放后居中放透明画布
async function generateForeground(foregroundPath, canvasSize, fgScale, outputPath) {
    const fgSize = Math.round(canvasSize * fgScale / 100);
    const { buffer: trimmedBuf, width: trimW, height: trimH } = await trimTransparent(foregroundPath);

    // 保持原始宽高比，按 fgSize 缩放
    let finalW, finalH;
    if (trimW >= trimH) {
        finalW = fgSize;
        finalH = Math.round(fgSize * trimH / trimW);
    } else {
        finalH = fgSize;
        finalW = Math.round(fgSize * trimW / trimH);
    }

    const fgBuffer = await sharp(trimmedBuf)
        .resize(finalW, finalH, { fit: 'fill' })
        .toBuffer();

    const offsetX = Math.round((canvasSize - finalW) / 2);
    const offsetY = Math.round((canvasSize - finalH) / 2);

    await sharp({
        create: {
            width: canvasSize,
            height: canvasSize,
            channels: 4,
            background: { r: 0, g: 0, b: 0, alpha: 0 }
        }
    })
    .composite([{ input: fgBuffer, top: offsetY, left: offsetX }])
    .png()
    .toFile(outputPath);
}

// 生成基础图标（缩放到画布尺寸）
async function generateBaseIcon(iconPath, canvasSize, outputPath) {
    await sharp(iconPath)
        .resize(canvasSize, canvasSize, { fit: 'contain' })
        .png()
        .toFile(outputPath);
}

// 写入自适应图标 XML
function writeAdaptiveIconXml(resDir) {
    const xml = `<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
  <foreground android:drawable="@mipmap/ic_launcher_foreground"/>
  <background android:drawable="@color/ic_launcher_background"/>
</adaptive-icon>`;
    writeFile(`${resDir}/mipmap-anydpi-v26/ic_launcher.xml`, xml);
}

// 写入图标背景色
function writeBackgroundColor(resDir, bgColor) {
    const xml = `<?xml version="1.0" encoding="utf-8"?>
<resources>
  <color name="ic_launcher_background">${bgColor}</color>
</resources>`;
    writeFile(`${resDir}/values/ic_launcher_background.xml`, xml);
}

async function main() {
    console.log('Copying frontend build to Android assets...');

    // 1. 复制 dist 到 Android assets
    copyDir(srcDist, destAssets);
    console.log(`  ✓ Copied ${srcDist} to ${destAssets}`);

    // 2. 生成 Android 图标
    const resDir = `${root}/gen/android/app/src/main/res`;
    const iconsDir = `${root}/icons`;
    const manifest = readManifest();

    const fgPath = path.resolve(iconsDir, manifest?.android_fg || 'android-fg.png');
    const iconPath = path.resolve(iconsDir, manifest?.default || 'app-icon.png');
    const fgScale = manifest?.android_fg_scale || 100;
    const bgColor = manifest?.bg_color || '#ffffff';

    console.log(`Generating Android icons (fg_scale=${fgScale}%, bg=${bgColor})...`);

    for (const [size, px] of Object.entries(MIPMAP_SIZES)) {
        const destDir = `${resDir}/mipmap-${size}`;

        // 自适应图标前景：缩放后居中放在透明画布上
        await generateForeground(fgPath, px, fgScale, `${destDir}/ic_launcher_foreground.png`);

        // 基础图标（非自适应设备回退用）
        await generateBaseIcon(iconPath, px, `${destDir}/ic_launcher.png`);

        // round 图标
        await generateBaseIcon(iconPath, px, `${destDir}/ic_launcher_round.png`);

        console.log(`  ✓ mipmap-${size} (${px}px)`);
    }

    // 自适应图标 XML
    writeAdaptiveIconXml(resDir);
    console.log(`  ✓ Adaptive icon XML`);

    // 图标背景色
    writeBackgroundColor(resDir, bgColor);
    console.log(`  ✓ Icon background color`);

    // 3. 复制用户自定义的 Android 配置文件
    const configDir = `${root}/android-config`;
    if (fs.existsSync(configDir)) {
        console.log('Copying Android config files...');
        copyDir(configDir, `${root}/gen/android`);
        console.log(`  ✓ Copied Android config files`);
    }

    console.log('\nAndroid resources ready!');
}

main().catch(err => {
    console.error('Build failed:', err);
    process.exit(1);
});
