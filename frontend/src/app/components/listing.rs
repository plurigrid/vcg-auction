use yew::prelude::*;

pub struct Listing;

impl Component for Listing {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                <div class="card">
                    <img src="..." class="card-img-top" alt="..." />
                    <div class="card-body">
                        <p class="card-text">
                            { "Some quick example text to build on the card title and make up the bulk of the card's content." }
                        </p>
                    </div>
                </div>
            </>
        }
    }
}
