import * as dir from 'node-dir';
import * as fs from 'fs';
import * as path from 'path';
import { DeploymentParameters } from './deployment_parameters';

/**
 * The purpose of this file is:
 * - (before this file runs) in deploy_all.sh, we deploy the Grapl stack, which
 *     outputs a `cdk-output.json`
 * - read in the "cdk-output.json" file to look for an apiUrl
 * - inject that into "edge_ux"'s files
 * - write those injected files to "edge_ux_package"
 * - (after this file runs) deploy the `edge_ux_package`
 *
 * Learn more at https://github.com/grapl-security/issue-tracker/issues/25
 */

function replaceInFile(
    toModify: string,
    replaceMap: Map<string, string>,
    outputFile: string
) {
    return fs.readFile(toModify, { encoding: 'utf8' }, (err, data) => {
        if (err) {
            return console.log(err);
        }

        let replaced = data;
        for (const [toReplace, replaceWith] of replaceMap.entries()) {
            replaced = replaced.split(toReplace).join(replaceWith);
        }

        if (outputFile) {
            fs.writeFile(
                outputFile,
                replaced,
                { encoding: 'utf8' },
                (err: any) => {
                    if (err) return console.log(err);
                }
            );
        } else {
            fs.writeFile(
                toModify,
                replaced,
                { encoding: 'utf8' },
                (err: any) => {
                    if (err) return console.log(err);
                }
            );
        }
    });
}

function getEdgeApiUrl(params: DeploymentParameters): string {
    /**
     * Read the 'cdk-output.json' as specified in `deploy_all.sh`
     */
    const outputFile = path.join(__dirname, '../cdk-output.json');
    const outputFileContents = JSON.parse(fs.readFileSync(outputFile, 'utf8'));
    // This looks like { DEPLOYMENT_NAME: { SOME_KEY: apiUrl } }
    const entryForDeployment = outputFileContents[params.stackName];
    if (entryForDeployment === undefined) {
        throw new Error(`Couldn't find an entry in cdk-output.json for ${params.stackName}`);
    }
    const apiUrl = Object.values(outputFileContents[params.stackName])[0];
    return apiUrl as string;
}

function createEdgeUxPackage(apiUrl: string) {
    const srcDir = path.join(__dirname, '../edge_ux_pre_replace/');
    const packageDir = path.join(__dirname, '../edge_ux_post_replace/');

    if (!fs.existsSync(packageDir)) {
        fs.mkdirSync(packageDir);
    }
    console.log("Replacing with /prod/");
    // TODO: Use base
    // https://developer.mozilla.org/en-US/docs/Web/HTML/Element/base
    const replaceMap = new Map();
    replaceMap.set(
        `script src="/`,
        `script src="/prod/`,
    );
    replaceMap.set(
        `link href="/`,
        `link href="/prod/`,
    );
    replaceMap.set(
        `"static/`,
        `"prod/static/`,
    );
    replaceMap.set(
        `"/static/`,
        `"/prod/static/`,
    );
    replaceMap.set(
        `"url(/static/`,
        `"url(/prod/static/`,
    );
    replaceMap.set(
        `"index.html`,
        `"prod/index.html`,
    );
    replaceMap.set('prod/prod/', 'prod/')

    dir.readFiles(
        srcDir,
        function (
            err: any,
            content: string | Buffer,
            filename: string,
            next: () => void
        ) {
            if (err) throw err;

            const targetDir = path
                .dirname(filename)
                .replace(srcDir, packageDir);

            if (!fs.existsSync(targetDir)) {
                fs.mkdirSync(targetDir, { recursive: true });
            }

            const newPath = filename.replace(
                srcDir,
                packageDir
            );

            replaceInFile(filename, replaceMap, newPath);
            next();
        },
        function (err: any, files: any) {
            console.warn(err);
            if (err) throw err;
        }
    );
}

const deploymentParameters = new DeploymentParameters();
const apiUrl = getEdgeApiUrl(deploymentParameters);
createEdgeUxPackage(apiUrl);
