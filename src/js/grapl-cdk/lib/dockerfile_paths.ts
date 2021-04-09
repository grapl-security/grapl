import * as path from "path";
import * as fs from "fs";

function ensureExists(dir: string) {
    if (!fs.existsSync(dir)) {
        throw new Error(`Couldn't resolve ${dir}`);
    }
}

// Akin to the build context in `docker-compose` files
export const SRC_DIR = path.join(__dirname, "../../../../src/");
ensureExists(SRC_DIR);

export const RUST_DOCKERFILE = "rust/Dockerfile";
export const PYTHON_DOCKERFILE = "python/Dockerfile";
