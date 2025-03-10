type LogoProps = {
  darkMode?: boolean;
};

export default function Logo(props: LogoProps) {
  return (
    <a href="https://www.gotcha.land" target="_blank">
      <img
        src="https://static.wixstatic.com/media/a56dc4_951625a6990f42b6a80975c7beabee2a~mv2.png/v1/fill/w_171,h_38,al_c,q_85,usm_0.66_1.00_0.01,enc_avif,quality_auto/HL_1.png"
        alt="Gotcha logo"
      />
    </a>
  );
}
