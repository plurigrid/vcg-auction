use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct Listing {
    pub title: String,
    pub description: String,
    pub current_bid: f64,
    pub image: String,
}

pub struct ListingGrid {
    listings: Vec<Listing>,
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub listings: Vec<Listing>,
}

impl Component for ListingGrid {
    type Message = ();
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            listings: ctx.props().listings.clone(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="row row-cols-1 row-cols-md-2 row-cols-lg-3 g-4">
                { for self.listings.iter().enumerate().map(|(index, listing)| self.view_listing(index, listing)) }
            </div>
        }
    }
}

impl ListingGrid {
    fn view_listing(&self, index: usize, listing: &Listing) -> Html {
        html! {
            <div key={index.to_string()} class="col">
                <div class="card h-100">
                    <img src={listing.image.clone()} class="card-img-top" alt={listing.title.clone()} />
                    <div class="card-body">
                        <h5 class="card-title">{ listing.title.clone() }</h5>
                        <p class="card-text">{ listing.description.clone() }</p>
                    </div>
                    <div class="card-footer">
                        <small class="text-muted">
                            { format!("Current Bid: ${:.2}", listing.current_bid) }
                        </small>
                    </div>
                </div>
            </div>
        }
    }
}
