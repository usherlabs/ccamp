var path = require('path');
var logger = require('morgan');
var express = require('express');
var createError = require('http-errors');
var cookieParser = require('cookie-parser');
const { default: mongoose } = require('mongoose');

var indexRouter = require('./routes/index');

require('dotenv').config();
var app = express();
var port = process.env.PORT || '3000';

const mongoDBconnectionString = process.env.MONGO_CONNECTION_STRING;

app.set('port', port);
// view engine setup
app.set('views', path.join(__dirname, 'views'));
app.set('view engine', 'jade');

app.use(logger('dev'));
app.use(express.json());
app.use(express.urlencoded({ extended: false }));
app.use(cookieParser());
app.use(express.static(path.join(__dirname, 'public')));

app.use('/', indexRouter);

// catch 404 and forward to error handler
app.use(function (req, res, next) {
	next(createError(404));
});

// error handler
app.use(function (err, req, res, next) {
	// set locals, only providing error in development
	res.locals.message = err.message;
	res.locals.error = req.app.get('env') === 'development' ? err : {};

	// render the error page
	res.status(err.status || 500);
	res.render('error');
});

// connect to mongoose using the string
mongoose
	.connect(mongoDBconnectionString, {
		useNewUrlParser: true,
		useUnifiedTopology: true,
		autoIndex: true, //make this also true
	})
	.then((db) => {
		console.log('sucesfully connected to DB. starting server...');
		// listen for http requests
		app.listen(port);
		console.log(`server started on port:${port}!!`);
	})
	.catch((err) => {
		console.log(err);
		console.log('there was an error connecting to mongodb database');
	});
