var express = require("express");
const dataModel = require("../schema/model");
var router = express.Router();

/* GET home page. */
router.post("/publish", async function (req, res, next) {
  const payload = req.body;
  console.log({
    payload,
  });
  const response = await dataModel.insertMany(payload);

  res.send(response);
});

/* GET home page. */
router.get("/query", async function (req, res, next) {
  const { from = 0 } = req.query;
  var timeStamp = new Date(+from).toISOString();

  const response = await dataModel.find({
    createdAt: { $gt: new Date(timeStamp) },
  });

  console.log[response]
  // res.json({ data: response, length: response.length, time: timeStamp });
  res.json(response);
});

module.exports = router;
