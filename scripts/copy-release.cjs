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

// 复制桌面可执行文件
function copyDesktopRelease() {
    console.log('\nCopying Desktop executable...');
    
    const releaseDir = path.join(rootDir, 'release', 'desktop');
    
    // 可能的构建输出目录
    const possibleDirs = [
        path.join(rootDir, 'src-tauri', 'target', 'release'),
        path.join(rootDir, 'src-tauri', 'target', 'x86_64-pc-windows-msvc', 'release'),
        path.join(rootDir, 'src-tauri', 'target', 'aarch64-apple-darwin', 'release'),
        path.join(rootDir, 'src-tauri', 'target', 'x86_64-unknown-linux-gnu', 'release'),
    ];
    
    // Windows: 查找 .exe 文件
    for (const dir of possibleDirs) {
        const exeFile = path.join(dir, 'reader.exe');
        if (fs.existsSync(exeFile)) {
            copyFile(exeFile, path.join(releaseDir, 'Reader.exe'));
            return;
        }
    }
    
    // Linux/macOS: 查找可执行文件
    for (const dir of possibleDirs) {
        const unixExe = path.join(dir, 'reader');
        if (fs.existsSync(unixExe)) {
            copyFile(unixExe, path.join(releaseDir, 'Reader'));
            return;
        }
    }
    
    console.log('  ⚠ No executable found');
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
