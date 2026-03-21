/**
 * 复制构建产物到 release 目录
 * 只复制最终安装包或可执行文件
 */

const fs = require('fs');
const path = require('path');

const rootDir = path.resolve(__dirname, '..');
const releaseDir = path.join(rootDir, 'release');

// 复制文件
function copyFile(src, dest) {
    const destDir = path.dirname(dest);
    if (!fs.existsSync(destDir)) {
        fs.mkdirSync(destDir, { recursive: true });
    }
    fs.copyFileSync(src, dest);
    console.log(`  ✓ ${path.basename(src)}`);
}

// 递归查找文件
function findFiles(dir, extensions) {
    const results = [];
    if (!fs.existsSync(dir)) return results;
    
    fs.readdirSync(dir).forEach(file => {
        const filePath = path.join(dir, file);
        const stat = fs.statSync(filePath);
        if (stat.isDirectory()) {
            results.push(...findFiles(filePath, extensions));
        } else if (extensions.some(ext => file.endsWith(ext))) {
            results.push(filePath);
        }
    });
    
    return results;
}

// 复制安卓 APK
function copyAndroidRelease() {
    console.log('\nCopying Android APK...');
    
    const apkDir = path.join(rootDir, 'src-tauri', 'gen', 'android', 'app', 'build', 'outputs', 'apk');
    const releaseDir = path.join(rootDir, 'release', 'android');
    
    const apks = findFiles(apkDir, ['.apk']);
    if (apks.length === 0) {
        console.log('  ⚠ No APK found');
        return;
    }
    
    apks.forEach(apk => {
        copyFile(apk, path.join(releaseDir, path.basename(apk)));
    });
}

// 复制桌面安装包
function copyDesktopRelease() {
    console.log('\nCopying Desktop installer...');
    
    const bundleDir = path.join(rootDir, 'src-tauri', 'target', 'release', 'bundle');
    const releaseDir = path.join(rootDir, 'release', 'desktop');
    
    const extensions = ['.msi', '.exe', '.deb', '.rpm', '.appimage', '.dmg'];
    const installers = findFiles(bundleDir, extensions);
    
    if (installers.length === 0) {
        console.log('  ⚠ No installer found');
        return;
    }
    
    installers.forEach(installer => {
        copyFile(installer, path.join(releaseDir, path.basename(installer)));
    });
}

// 主函数
function main() {
    const platform = process.argv[2];
    
    console.log('Copying release files...');
    
    if (platform === 'android') {
        copyAndroidRelease();
    } else if (platform === 'desktop') {
        copyDesktopRelease();
    } else if (platform === 'all') {
        copyAndroidRelease();
        copyDesktopRelease();
    } else {
        console.log('Usage: node copy-release.cjs [android|desktop|all]');
        process.exit(1);
    }
    
    console.log('\nDone!');
}

main();
