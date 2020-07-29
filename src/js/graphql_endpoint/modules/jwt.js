const jwt = require('jsonwebtoken');
const AWS = require('aws-sdk')

const IS_LOCAL = (process.env.IS_LOCAL == 'True') || null;  // get this from environment
const JWT_SECRET_ID = process.env.JWT_SECRET_ID;  // get this from environment

// Acts as a local cache of the secret so we don't have to refetch it every time
let JWT_SECRET = "";


const secretsmanager = new AWS.SecretsManager({
    apiVersion: '2017-10-17',
    region: IS_LOCAL ? 'us-east-1' : undefined,
    accessKeyId: IS_LOCAL ? 'dummy_cred_aws_access_key_id' : undefined,
    secretAccessKey: IS_LOCAL ? 'dummy_cred_aws_secret_access_key' : undefined,
    endpoint: IS_LOCAL ? 'http://secretsmanager.us-east-1.amazonaws.com:4566': undefined,
});

const fetchJwtSecret = async () => {
    const getSecretRes = await secretsmanager.getSecretValue({
        SecretId: JWT_SECRET_ID,
    }).promise();

    return getSecretRes.SecretString;
}

// Prefetch the secret
(async () => {
    try {
        if (!JWT_SECRET) {
            JWT_SECRET = await fetchJwtSecret()
            .catch((e) => console.warn(e));
        }
    } catch (e) {
        console.error(e);
        // Deal with the fact the chain failed
    }
})();

const verifyToken = async (jwtToken) => {
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


const validateJwt = async (req, res, next) => {
    const headers = req.headers;
    encoded_jwt = null

    if (!headers.cookie) {
        console.log("Missing cookie: ", headers)
        return res.sendStatus(401) // if there isn't any token
    }

    for (cookie of headers.cookie.split(';')) {
        if (cookie.startsWith('grapl_jwt=')) {
            encoded_jwt = cookie.split('grapl_jwt=')[1].trim()
            break
        }
    }

    if (encoded_jwt == null) return res.sendStatus(401) // if there isn't any token

    if (await verifyToken(encoded_jwt) !== null) {
        next() 
    } else {
        return res.sendStatus(403)
    }
}

module.exports = {
    validateJwt,
    verifyToken,
}