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

// 复制文件（如果源文件存在）
function copyFileIfExists(src, dest) {
    if (fs.existsSync(src)) {
        const destDir = path.dirname(dest);
        if (!fs.existsSync(destDir)) {
            fs.mkdirSync(destDir, { recursive: true });
        }
        fs.copyFileSync(src, dest);
        return true;
    }
    return false;
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
        copyFileIfExists(
            `${root}/icons/${size}/ic_launcher.png`,
            `${mipmapDir}/ic_launcher.png`
        );
        
        // 复制 ic_launcher_round.png
        copyFileIfExists(
            `${root}/icons/${size}/ic_launcher_round.png`,
            `${mipmapDir}/ic_launcher_round.png`
        );
        
        // 复制 ic_launcher_foreground.png
        copyFileIfExists(
            `${root}/icons/${size}/ic_launcher_foreground.png`,
            `${mipmapDir}/ic_launcher_foreground.png`
        );
    });
    console.log(`  ✓ Copied icons to mipmap directories`);
    
    // 3. 复制用户自定义的 Android 配置文件
    const configDir = `${root}/android-config`;
    if (fs.existsSync(configDir)) {
        console.log('Copying Android config files...');
        copyDir(configDir, `${root}/gen/android`);
        console.log(`  ✓ Copied Android config files`);
    }
    
    console.log('\nAndroid resources ready!');
}

main();
