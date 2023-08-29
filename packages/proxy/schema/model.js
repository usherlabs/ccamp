const { default: mongoose } = require("mongoose");

const RelayDataSchema = mongoose.Schema(
  {
    event_name: String,
    canister_id: String,
    account: String,
    amount: Number,
    chain: String,
    token: String,
  },
  { timestamps: true }
);

const dataModel = mongoose.model("RelayData", RelayDataSchema);
module.exports = dataModel;
