use std::future::Future;
use std::sync::Arc;
use std::pin::Pin;

use warp::Filter;
use warp::http::StatusCode;

type PinFuture<'a, T> = Pin<Box<dyn 'a + Send + Future<Output=Result<T, ()>>>>;

trait Provider: Send + Sync {
    fn can_see<'a>(&'a self, c: &'a Item) -> PinFuture<'a, bool>;
}

struct Item;

impl Item {
    pub fn is_deleted(&self) -> bool {
        false
    }
}

async fn write_items<'a, T, P>(p: &P, items: T) -> Result<(), ()>
    where P: Provider,
        T: IntoIterator<Item=&'a Item>
{
    for item in items {
        if p.can_see(item).await? {
            // Here we'd serialize the item, but this is unnecessary to cause
            // the lifetime issue.
        }
    }
    Ok(())
}

fn demo<T: Provider>(provider: Arc<T>) -> impl Filter<Extract=(impl warp::Reply,)> {
    warp::get()
    .and(warp::path::param::<String>())
    .and(warp::any().map(move || provider.clone()))
    .and_then(|_id, provider: Arc<T>| async move {
        let items = vec![Item];

        let items = items.iter().filter(|item| !item.is_deleted());

        match write_items(&*provider, items).await {
            Ok(_) => {},
            Err(_) => return Ok(StatusCode::INTERNAL_SERVER_ERROR),
        };

        Ok::<StatusCode, warp::Rejection>(StatusCode::OK)
    })
}

struct ExampleProvider;

impl Provider for ExampleProvider {
    fn can_see<'a>(&'a self, _c: &'a Item) -> PinFuture<'a, bool> {
        Box::pin(std::future::ready(Ok(true)))
    }
}

fn main() {
    demo(Arc::new(ExampleProvider));
}
