const AWS = require('aws-sdk');
AWS.config.update({region:'us-east-1'});

let apigateway = new AWS.APIGateway();


var params = {
  limit: 1000,
};

apigateway.getRestApis(params, function(err, data) {
  for (const item of data.items) {
    console.log(item.endpointConfiguration)
  }
});

