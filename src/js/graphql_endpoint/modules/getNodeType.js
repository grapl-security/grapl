const AWS = require("aws-sdk");

export const getNodeType = (nodeUid) => {
	// make a call to dynamo db with the node and get back it's node type
	AWS.config.update({ region: "us-east-1" });

	// Create the DynamoDB service object
	const ddb = new AWS.DynamoDB({ apiVersion: "2012-08-10" });

	const params = {
		TableName: dynamodb.Table(os.environ["DEPLOYMENT_NAME"] + "-grapl_schema_table"),
		Key: {
			KEY_NAME: { node_type: nodeUid },// is this supposed to be a primary key? 
		},
		ProjectionExpression: "uid", //ing that identifies the attributes that you want
	};

	// Call DynamoDB to read the item from the table
	ddb.getItem(params, function (err, data) {
		if (err) {
			console.log("Error", err);
		} else {
			console.log("Success", data.Item);
		}
	});
};