import * as express from "express";
import jwt = require('jsonwebtoken');
import AWS = require('aws-sdk');

const IS_LOCAL = (process.env.IS_LOCAL == 'True') || null;  // get this from environment
const JWT_SECRET_ID = process.env.JWT_SECRET_ID;  // get this from environment

// Acts as a local cache of the secret so we don't have to refetch it every time
let JWT_SECRET: string = "";


const secretsmanager = new AWS.SecretsManager({
    apiVersion: '2017-10-17',
    region: IS_LOCAL ? process.env.AWS_REGION : undefined,
    accessKeyId: IS_LOCAL ? process.env.SECRETSMANAGER_ACCESS_KEY_ID : undefined,
    secretAccessKey: IS_LOCAL ? process.env.SECRETSMANAGER_ACCESS_KEY_SECRET : undefined,
    endpoint: IS_LOCAL ? process.env.SECRETSMANAGER_ENDPOINT : undefined,
});

async function fetchJwtSecret(): Promise<string> {
    console.log("JWT_SECRET_ID: ", JWT_SECRET_ID);
    const getSecretRes = await secretsmanager.getSecretValue({
        SecretId: JWT_SECRET_ID,
    }).promise();

    return getSecretRes.SecretString;
}

// Prefetch the secret
(async () => {
    try {
        if (!JWT_SECRET) {
            JWT_SECRET = await fetchJwtSecret();
        }
    } catch (e) {
        console.error(e);
    }
})();

export async function verifyToken(jwtToken: string) {
    if (!JWT_SECRET) {
        JWT_SECRET = await fetchJwtSecret();
    }
    try {
        return jwt.verify(jwtToken, JWT_SECRET, {
            algorithms: ['HS256']
        });
    } catch(e) {
        console.log('JWT failed with:',e);
        return null;
    }
};


export async function validateJwt(
    req: express.Request, 
    res: express.Response, 
    next: express.NextFunction
): Promise<express.Response> {
    const headers = req.headers;
    let encoded_jwt = null

    if (!headers.cookie) {
        console.log("Missing cookie: ", headers)
        return res.sendStatus(401) // if there isn't any token
    }

    for (const _cookie of headers.cookie.split(';')) {
        const cookie = _cookie.trim();
        if (cookie.startsWith('grapl_jwt=')) {
            encoded_jwt = cookie.split('grapl_jwt=')[1].trim()
            break
        }
    }

    if (encoded_jwt == null) {
        console.warn('Missing jwt from cookie: ', headers)
        return res.sendStatus(401)
    }

    if (await verifyToken(encoded_jwt) !== null) {
        next() 
    } else {
        return res.sendStatus(403)
    }
}