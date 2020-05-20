const jwt = require('jsonwebtoken');

const SECRET_KEY = process.env.JWT_SECRET_KEY;  // get this from environment

const verifyToken = (jwtToken) => {
    try {
        return jwt.verify(jwtToken, SECRET_KEY, {
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