use super::*;

//------------------------------------------------------

pub trait IntoDiskRepr<T> {

}

pub trait FromDiskRepr<'a, T> {
    
}

impl<'a, T: Serialize + Deserialize<'a>> DiskRepr for Schedule<T>
where T: IntoDiskRepr<T> + FromDiskRepr<'a, T> {}

#[derive(Serialize, Deserialize)]
pub struct JobsSerialized<T>(Vec<T>);
impl<T> DiskRepr for JobsSerialized<T> {}

/// Partially serialize into raw bytes, 
impl<'a, T, R> IntoDiskRepr<JobsSerialized<T>> for Schedule<T, R>
where T: Serialize + Deserialize<'a> {
    fn into_raw_repr(self) -> JobsSerialized<T> {
        JobsSerialized(self.jobs.into_iter().map(|x| x.clone()).collect())
    }
}

/// Partially Deserialization raw bytes into original
impl<'a, T, R> FromDiskRepr<'a, JobsSerialized<T>> for Schedule<T, R> where T: Deserialize<'a> {
    fn from_raw_repr(&mut self, buf: &'a [u8], input: Mode) -> Result<Self, Box<std::error::Error>> {
        self.jobs = bincode::deserialize::<JobsSerialized<T>>(buf)?.0
            .into_iter()
            .map(|x| Arc::new(x))
            .collect();
        Ok(Self)
    }
}

pub async fn post_process<R>(rx: &mut mpsc::Receiver<R>, post_hooks: &[R]) -> Option<R> 
where 
    R: Consumer<R> + Copy
{
    if let Some(mut item) = rx.recv().await {
        for hook in post_hooks {
            match hook.process(item).await {
                Some(new_item) => {
                    item = new_item;
                }
                None => return None
            }
        }
        return Some(item)
    }
    None
}
