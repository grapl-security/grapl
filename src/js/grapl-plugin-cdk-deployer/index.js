"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (_) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
exports.__esModule = true;
var AWS = require("aws-sdk");
var adm_zip_1 = require("adm-zip");
var fsPromises = require('fs').promises;
// Set the region
// Create SQS service object
var sqs = new AWS.SQS();
var s3 = new AWS.S3();
// Replace with your accountid and the queue name you setup
var accountId = '131275825514';
var queueName = 'test';
var queueUrl = "https://sqs.us-east-1.amazonaws.com/" + accountId + "/" + queueName;
// Setup the receiveMessage parameters
var params = {
    QueueUrl: queueUrl,
    MaxNumberOfMessages: 1,
    WaitTimeSeconds: 10
};
var main = function () { return __awaiter(void 0, void 0, void 0, function () {
    var Messages, msg, record, bucket, key, s3Params, packaged_payload, zip;
    return __generator(this, function (_a) {
        switch (_a.label) {
            case 0: return [4 /*yield*/, sqs.receiveMessage(params).promise()];
            case 1:
                Messages = (_a.sent()).Messages;
                if (!Messages) {
                    return [2 /*return*/];
                }
                msg = JSON.parse(Messages[0].Body || "{}");
                record = msg["Records"][0];
                bucket = record["s3"]["bucket"]["name"];
                key = record["s3"]["object"]["key"];
                s3Params = {
                    Bucket: bucket,
                    Key: key
                };
                return [4 /*yield*/, s3.getObject(s3Params).promise()];
            case 2:
                packaged_payload = _a.sent();
                return [4 /*yield*/, fsPromises.writeFile('./package.zip', packaged_payload)];
            case 3:
                _a.sent();
                zip = new adm_zip_1["default"]("./my_file.zip");
                zip.extractAllTo(/*target path*/ "./zips/", /*overwrite*/ true);
                return [2 /*return*/];
        }
    });
}); };
(function () { return __awaiter(void 0, void 0, void 0, function () {
    return __generator(this, function (_a) {
        switch (_a.label) {
            case 0: return [4 /*yield*/, main()];
            case 1:
                _a.sent();
                _a.label = 2;
            case 2: return [3 /*break*/, 0];
            case 3: return [2 /*return*/];
        }
    });
}); })()["catch"](function (e) {
    // Deal with the fact the chain failed
    console.error(e);
});
