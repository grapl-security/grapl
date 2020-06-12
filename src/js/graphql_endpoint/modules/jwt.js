const jwt = require('jsonwebtoken');
const AWS = require('aws-sdk')
var secretsmanager = new AWS.SecretsManager({apiVersion: '2017-10-17'});

const IS_LOCAL = (process.env.IS_LOCAL == 'True') || null;  // get this from environment
const JWT_SECRET_ID = process.env.JWT_SECRET_ID;  // get this from environment
let JWT_SECRET = "";

const params = {
    SecretId: JWT_SECRET_ID,
};

if (!IS_LOCAL) {
    secretsmanager.getSecretValue(params, (err, data) => {
        if (err) {
            console.log(err, err.stack)
        } // an error occurred
        else {
            console.log('Retriever secret with version: ', data.VersionId);
            JWT_SECRET = data.SecretString;
        }
    });
}


const verifyToken = (jwtToken) => {
    try {
        return jwt.verify(jwtToken, JWT_SECRET, {
            algorithms: ['HS256']
        });
    } catch(e) {
        console.log('JWT failed with:',e);
        return null;
    }
};


const validateJwt = (req, res, next) => {
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

    if (verifyToken(encoded_jwt) !== null) {
        next() 
    } else {
        return res.sendStatus(403)
    }
}

module.exports = {
    validateJwt,
    verifyToken,
}