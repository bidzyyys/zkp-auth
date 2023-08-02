const chai = require("chai");
const chaiHttp = require("chai-http");

chai.use(chaiHttp);
const expect = chai.expect;

const SERVER_URL = "http://localhost:8080";
const USERNAME = "bidzyyys";
const SECRET = 9;
const CHALLENGE_K = 27;

// eslint-disable-next-line no-undef
describe("Happy Path", () => {
	// eslint-disable-next-line no-undef
	it("Should reject login without user registration", (done) => {
		chai.request(SERVER_URL)
			.post("/login")
			.send({ user: USERNAME, x: SECRET, k: CHALLENGE_K })
			.end((err, res) => {
				expect(res).to.have.status(406);
				// eslint-disable-next-line no-unused-vars
				done();
			});
	});

	let y1 = null;
	let y2 = null;

	// eslint-disable-next-line no-undef
	it("Should calculate registration data", (done) => {
		chai.request(SERVER_URL)
			.post("/register/calculate")
			.send({ user: USERNAME, x: SECRET })
			.end((err, res) => {
				expect(err).to.be.null;
				expect(res).to.be.json;
				expect(res).to.have.status(201);
				expect(res.body).to.have.property("y1");
				expect(res.body).to.have.property("y2");
				y1 = res.body.y1;
				y2 = res.body.y2;

				// eslint-disable-next-line no-unused-vars
				done();
			});
	});

	// eslint-disable-next-line no-undef
	it("Should register user", (done) => {
		chai.request(SERVER_URL)
			.post("/register")
			.send({ username: USERNAME, y1: y1, y2: y2 })
			.end((err, res) => {
				expect(err).to.be.null;
				expect(res).to.have.status(201);
				// eslint-disable-next-line no-unused-vars
				done();
			});
	});

	// eslint-disable-next-line no-undef
	it("Should reject repeated user registration", (done) => {
		chai.request(SERVER_URL)
			.post("/register")
			.send({ username: USERNAME, y1: 1, y2: 2 })
			.end((err, res) => {
				expect(res).to.have.status(406);
				// eslint-disable-next-line no-unused-vars
				done();
			});
	});

	// eslint-disable-next-line no-undef
	it("Should accept valid login attempt", (done) => {
		chai.request(SERVER_URL)
			.post("/login")
			.send({ user: USERNAME, x: SECRET, k: CHALLENGE_K })
			.end((err, res) => {
				expect(err).to.be.null;
				expect(res).to.have.status(200);
				expect(res).to.be.json;
				expect(res.body)
					.to.have.property("session_id")
					.equal("test session");
				//eslint-disable-next-line no-unused-vars
				done();
			});
	});
});
