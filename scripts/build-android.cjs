/**
 * 构建 Android 资源脚本
 * 将前端构建产物和图标复制到 Android 项目目录
 */

const fs = require('fs');
const path = require('path');

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

// 复制文件（如果源文件存在且目标文件不存在）
function copyFile(src, dest) {
    if (fs.existsSync(src) && !fs.existsSync(dest)) {
        fs.copyFileSync(src, dest);
    }
}

function main() {
    console.log('Copying frontend build to Android assets...');
    
    // 1. 复制 dist 到 Android assets
    copyDir(srcDist, destAssets);
    console.log(`  ✓ Copied ${srcDist} to ${destAssets}`);
    
    // 2. 复制图标到各分辨率 mipmap 目录
    const sizes = ['mdpi', 'hdpi', 'xhdpi', 'xxhdpi', 'xxxhdpi'];
    sizes.forEach(size => {
        const mipmapDir = `${root}/gen/android/app/src/main/res/mipmap-${size}`;
        
        // 复制 ic_launcher.png
        copyFile(
            `${root}/icons/${size}/ic_launcher.png`,
            `${mipmapDir}/ic_launcher.png`
        );
        
        // 复制 ic_launcher_round.png
        copyFile(
            `${root}/icons/${size}/ic_launcher_round.png`,
            `${mipmapDir}/ic_launcher_round.png`
        );
        
        // 复制 ic_launcher_foreground.png
        copyFile(
            `${root}/icons/${size}/ic_launcher_foreground.png`,
            `${mipmapDir}/ic_launcher_foreground.png`
        );
    });
    console.log(`  ✓ Copied icons to mipmap directories`);
    
    console.log('\nAndroid resources ready!');
}

main();
