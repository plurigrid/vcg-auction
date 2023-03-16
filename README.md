<img width="546" alt="cats-waiting-to-vote" src="https://user-images.githubusercontent.com/44031/215276751-5ed09b08-23b2-420e-96d0-732b911612ad.png">

This represents a contract for a "reverse" VCG (vickery-clark-groves) auction also known as a "second price auction". This means that a number of bidders will bid, and the
lowest bidder will win the auction. The winner will pay the second lowest bid.

The contract runs one auction at a time.

Each auction is instantiated with a fixed number of bidders. Once the target number of bidders has been satisfied, the auction may be closed and a winner selected.

Authorizing bidders based on predetermined addresses will be a necessary but later feature of this contract.


