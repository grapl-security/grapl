import * as path from 'path';
import * as fs from 'fs';

function ensureExists(dir: string) {
    if(!fs.existsSync(dir)) {
        throw new Error(`Couldn't resolve ${dir}`);
    }
}

export const PYTHON_DIR = path.join(__dirname, '../../../../src/python/');
export const RUST_DIR = path.join(__dirname, '../../../../src/rust/');

for (const dir of [PYTHON_DIR, RUST_DIR]) {
   ensureExists(dir);
}