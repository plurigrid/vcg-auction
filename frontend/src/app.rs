// src/gui/app.rs

mod components {
    pub mod header;
    pub mod listing;
    pub mod listing_grid;
}

use components::{
    header::Header,
    listing_grid::{Listing, ListingGrid},
};
use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    let listings = vec![
        Listing {
            title: "Luxury Apartment in Manhattan".into(),
            description: "This spacious and elegant apartment is located in the heart of Manhattan, offering breathtaking views of the city skyline. With 3 bedrooms and 2 bathrooms, this apartment can accommodate up to 6 guests. Amenities include a fully equipped kitchen, high-speed internet, and cable TV.".into(),
            current_bid: 350.0,
            image: "https://picsum.photos/id/237/600/400".into(),
        },
        // Add other listings here
    ];

    html! {
        <>
            <Header />
            <ListingGrid listings={listings} />
        </>
    }
}
