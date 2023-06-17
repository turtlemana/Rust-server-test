import Image from 'next/image';
import axios from 'axios';

export default async function Home() {
  const res = await axios.get('http://127.0.0.1:7878/api/ETH-USD');
  const data = res.data;
  console.log(data);
  // console.log(data);
  return (
    <main>
      <div>{data.chart.toString()}</div>
    </main>
  );
}
